#![feature(int_roundings)]

mod boilerplate;
mod movement;

use std::{num::NonZeroU32, time::Instant};
use glam::Vec4;
use movement::*;
use boilerplate::*;

use crevice::std430::AsStd430;
use shader::ShaderConstants;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferDescriptor, BufferUsages, Extent3d, PipelineLayoutDescriptor, PushConstantRange, ShaderModuleDescriptorSpirV, ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use winit::{event::{ElementState, Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowBuilder}};
const SHADER: &[u8] = include_bytes!(env!("shader.spv"));


fn main() {
    env_logger::builder().filter(Some("wgpu_core"), log::LevelFilter::Warn).filter(Some("wgpu_hal"), log::LevelFilter::Warn).filter_level(log::LevelFilter::Debug).init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(setup(&window));     

    let module = unsafe { 
        state.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: None,
            source: std::borrow::Cow::Borrowed(bytemuck::cast_slice::<u8, u32>(SHADER)),
        })
    };

    let bind_group_layout = state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: state.config.format, view_dimension: TextureViewDimension::D2 },
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
    
    let buffer = state.device.create_buffer(&BufferDescriptor {
        label: None,
        size: std::mem::size_of::<ShaderConstants>() as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false
    });

    let layout = state.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&layout),
        module: &module,
        entry_point: "test"
    });

    let mut instant = Instant::now();
    let mut movement = Movement::default();
    movement.position.y = 10f32;

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    window.set_cursor_visible(false);

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => control_flow.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                state.config.width = new_size.width;
                state.config.height = new_size.height;
                state.surface.configure(&state.device, &state.config);
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let State {
                    instance,
                    surface,
                    adapter,
                    device,
                    queue,
                    config,
                } = &state;


                let delta = (Instant::now() - instant).as_secs_f32();
                movement.update(window.inner_size().width as f32 / window.inner_size().height as f32, delta);
                instant = Instant::now();

                let constants = ShaderConstants {
                    //time: (Instant::now() - instant).as_secs_f32(),
                    proj_matrix: movement.proj_matrix,
                    view_matrix: movement.view_matrix,
                    position: Vec4::from((movement.position, 0f32)),
                    width: window.inner_size().width as f32,
                    height: window.inner_size().height as f32,
                };

                let data = constants.as_std430();
                let aaa = bytemuck::bytes_of(&data);
                queue.write_buffer(&buffer, 0, aaa);


                let tex = surface.get_current_texture().unwrap();
                let view = tex.texture.create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: None,
                    aspect: TextureAspect::All,
                    base_mip_level: 0, mip_level_count: NonZeroU32::new(1), base_array_layer: 0, array_layer_count: None });
                
                let mut encoder = device.create_command_encoder(&Default::default());

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&view),
                    }, BindGroupEntry {
                        binding: 1,
                        resource: buffer.as_entire_binding(),
                    }],
                });
                
                let mut _compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                _compute_pass.set_pipeline(&pipeline);
                _compute_pass.set_bind_group(0, &bind_group, &[]);

                


                let x = window.inner_size().width.div_ceil(32);
                let y = window.inner_size().height.div_ceil(32);

                _compute_pass.dispatch_workgroups(x, y, 1);
                drop(_compute_pass);
                
                queue.submit([encoder.finish()]);

                tex.present();
                window.request_redraw()
            },

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                    movement.key_pressed(event.physical_key, event.state);

                    if event.physical_key == PhysicalKey::Code(KeyCode::F5) && event.state == ElementState::Pressed {
                        if window.fullscreen().is_none() {
                            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        } else {
                            window.set_fullscreen(None);
                        }
                    }
                },
                WindowEvent::MouseInput { device_id, state, button } => {},
                _ => {}
            },

            Event::DeviceEvent { device_id, event } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => movement.mouse_delta(delta.0 as f32, delta.1 as f32),
                winit::event::DeviceEvent::MouseWheel { delta } => {},
                _ => {}
            }

            _ => (),
        }
    }).unwrap();
}
