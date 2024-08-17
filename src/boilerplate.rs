
use wgpu::{Adapter, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureFormat, TextureUsages};
use winit::window::Window;

pub struct State {
    pub instance: Instance,
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

pub async fn setup(window: &Window) -> State {
    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::VULKAN,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.unwrap();

    let (device, queue) = adapter.request_device(
        &DeviceDescriptor {
            features: Features::default() | Features::SPIRV_SHADER_PASSTHROUGH | Features::PUSH_CONSTANTS | Features::PUSH_CONSTANTS,
            limits: wgpu::Limits {
                max_push_constant_size: 128,
                ..Default::default()
            },
            label: None,
        },
        None,
    ).await.unwrap();

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps.formats.iter()
        .find(|f| f.describe().srgb)
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::Immediate,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };

    log::debug!("Format Type: {:?}", surface_format);

    surface.configure(&device, &config);

    State {
        instance,
        surface,
        adapter,
        device,
        queue,
        config,
    }
}