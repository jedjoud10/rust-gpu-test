#![feature(int_roundings)]
use input::Input;
use shared::*;
mod boilerplate;
mod movement;
mod input;

use std::{mem::size_of, num::NonZeroU32, time::Instant};
use glam::Vec4;
use movement::*;
use boilerplate::*;
use crevice::std430::AsStd430;
use wgpu::{util::make_spirv_raw, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferDescriptor, BufferUsages, Extent3d, PipelineLayoutDescriptor, PushConstantRange, ShaderModuleDescriptorSpirV, ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use winit::{event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowBuilder}};
const RAYMARCH: &[u8] = include_bytes!(env!("raymarch.spv"));
const BLIT: &[u8] = include_bytes!(env!("blit.spv"));


fn main() {
    env_logger::builder().filter(Some("wgpu_core"), log::LevelFilter::Warn).filter(Some("wgpu_hal"), log::LevelFilter::Warn).filter_level(log::LevelFilter::Debug).init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(setup(&window));    

    println!("{}", env!("raymarch.spv")); 



    let raymarch_module = unsafe { 
        state.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: Some("raymarch module"),
            source: make_spirv_raw(RAYMARCH),
        })
    };
    
    let blit_module = unsafe { 
        state.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: Some("blit module"),
            source: make_spirv_raw(BLIT),
        })
    };

    let bind_group_layout = state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("raymarch bind group layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        }
        ],
    });

    let bind_group_layout_blit = state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("blit bind group layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture { access: StorageTextureAccess::ReadOnly, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: state.config.format, view_dimension: TextureViewDimension::D2 },
            count: None,
        }
        ],
    });

    let size_reduction = 2;

    let mut src_output = state.device.create_texture(&TextureDescriptor {
        label: Some("output raymarch texture"),
        size: Extent3d { width: state.config.width / size_reduction, height: state.config.height / size_reduction, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING,
        view_formats: &[] }
    );
    
    let buffer = state.device.create_buffer(&BufferDescriptor {
        label: Some("raymarch uniform buffer"),
        size: std::mem::size_of::<RaymarchParams>() as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false
    });

    let layout = state.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("raymarch pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("raymarch pipeline"),
        layout: Some(&layout),
        module: &raymarch_module,
        entry_point: "main"
    });

    let blit_layout = state.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("blit pipeline layout"),
        bind_group_layouts: &[&bind_group_layout_blit],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..size_of::<u32>() as u32,
        }],
    });

    let blit_pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("blit pipeline"),
        layout: Some(&blit_layout),
        module: &blit_module,
        entry_point: "main"
    });


    let mut instant = Instant::now();
    let mut movement = Movement::default();
    movement.position.y = 10f32;

    let mut input = Input::default();
    window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    window.set_cursor_visible(false);

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => control_flow.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                state.config.width = new_size.width;
                state.config.height = new_size.height;
                state.surface.configure(&state.device, &state.config);

                src_output = state.device.create_texture(&TextureDescriptor {
                    label: Some("output raymarch texture"),
                    size: Extent3d { width: state.config.width / size_reduction, height: state.config.height / size_reduction, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8Unorm,
                    usage: TextureUsages::STORAGE_BINDING,
                    view_formats: &[] }
                );
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let State {
                    instance,
                    surface,
                    adapter,
                    device,
                    queue,
                    config,
                    ..
                } = &state;

                if input.get_button(KeyCode::F5).pressed() {
                    if window.fullscreen().is_none() {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    } else {
                        window.set_fullscreen(None);
                    }
                }

                let delta = (Instant::now() - instant).as_secs_f32();
                movement.update(&input, window.inner_size().width as f32 / window.inner_size().height as f32, delta);
                instant = Instant::now();

                let constants = RaymarchParams {
                    //time: (Instant::now() - instant).as_secs_f32(),
                    proj_matrix: movement.proj_matrix,
                    view_matrix: movement.view_matrix,
                    position: Vec4::from((movement.position, 0f32)),
                    width: (window.inner_size().width / size_reduction) as f32,
                    height: (window.inner_size().height / size_reduction) as f32,
                };

                let data = constants.as_std430();
                let raw = bytemuck::bytes_of(&data);
                queue.write_buffer(&buffer, 0, raw);


                let src_view = src_output.create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: None,
                    aspect: TextureAspect::All,
                    base_mip_level: 0, mip_level_count: NonZeroU32::new(1), base_array_layer: 0, array_layer_count: None }
                );

                let surface_texture = state.surface.get_current_texture().unwrap();
                let dst_view = surface_texture.texture.create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: None,
                    aspect: TextureAspect::All,
                    base_mip_level: 0, mip_level_count: NonZeroU32::new(1), base_array_layer: 0, array_layer_count: None });
                
                let mut encoder = device.create_command_encoder(&Default::default());

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("raymarch bind group"),
                    layout: &bind_group_layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    }, BindGroupEntry {
                        binding: 1,
                        resource: buffer.as_entire_binding(),
                    }],
                });

                
                let bind_group_blit = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("blit bind group"),
                    layout: &bind_group_layout_blit,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    }, BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&dst_view),
                    }],
                });
                
                let mut _compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                _compute_pass.set_pipeline(&pipeline);
                _compute_pass.set_bind_group(0, &bind_group, &[]);

                let x = window.inner_size().width.div_ceil(32 * size_reduction);
                let y = window.inner_size().height.div_ceil(32 * size_reduction);
                _compute_pass.dispatch_workgroups(x, y, 1);

                _compute_pass.set_pipeline(&blit_pipeline);
                _compute_pass.set_bind_group(0, &bind_group_blit, &[]);
                _compute_pass.set_push_constants(0, bytemuck::bytes_of(&size_reduction));
                let x = window.inner_size().width.div_ceil(32);
                let y = window.inner_size().height.div_ceil(32);
                _compute_pass.dispatch_workgroups(x, y, 1);

                drop(_compute_pass);
                
                queue.submit([encoder.finish()]);

                
                surface_texture.present();
                window.request_redraw();
                input::update(&mut input);
            },

            Event::WindowEvent { event, .. } => {
                input::window_event(&mut input, &event);
            }

            Event::DeviceEvent { device_id, event } => {
                input::device_event(&mut input, &event);
            }

            _ => (),
        }
    }).unwrap();
}
