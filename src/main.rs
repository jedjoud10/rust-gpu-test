#![feature(int_roundings)]

mod boilerplate;
use std::{num::NonZeroU32, time::Instant};
use boilerplate::*;

use shader::ShaderConstants;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType::StorageTexture, Extent3d, PipelineLayoutDescriptor, PushConstantRange, ShaderModuleDescriptorSpirV, ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};
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
            ty: StorageTexture { access: StorageTextureAccess::WriteOnly, format: state.config.format, view_dimension: TextureViewDimension::D2 },
            count: None,
        }],
    });

    let layout = state.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::COMPUTE,
            range: 0..4,
        }],
    });

    let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&layout),
        module: &module,
        entry_point: "main"
    });

    let instant = Instant::now();
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
                    }],
                });
                
                let mut _compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                _compute_pass.set_pipeline(&pipeline);
                _compute_pass.set_bind_group(0, &bind_group, &[]);

                let constants = ShaderConstants {
                    time: (Instant::now() - instant).as_secs_f32(),
                };

                let bytes = bytemuck::bytes_of(&constants);
                _compute_pass.set_push_constants(0, bytes);

                let x = window.inner_size().width.div_ceil(32);
                let y = window.inner_size().height.div_ceil(32);

                _compute_pass.dispatch_workgroups(x, y, 1);
                drop(_compute_pass);
                
                queue.submit([encoder.finish()]);

                tex.present();
                window.request_redraw()
            },

            _ => (),
        }
    }).unwrap();
}
