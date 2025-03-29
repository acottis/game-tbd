use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl State {
    fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let window_size = window.inner_size();
        let instance =
            wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let (adapter, device, queue) =
            pollster::block_on(request_adapter(&instance));

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        println!("{device:#?}");
        println!("{queue:#?}");
        println!("{surface:#?}");

        Self {
            window,
            surface,
            surface_config,
            device,
            queue,
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl App {
    fn init(&mut self, window: Window) {
        self.state = Some(State::new(window))
    }

    fn render(&mut self) {
        let state = self.state.as_ref().unwrap();
        let surface_texture = state.surface.get_current_texture().unwrap();

        let mut encoder =
            state.device.create_command_encoder(&Default::default());

        let render_pass_color_attachment = wgpu::RenderPassColorAttachment {
            view: &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                store: wgpu::StoreOp::Store,
            },
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(render_pass_color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        encoder.begin_render_pass(&render_pass_descriptor);

        state.queue.submit([encoder.finish()]);
        state.window.pre_present_notify();
        surface_texture.present();
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width * size.height == 0 {
            return;
        }

        let state = self.state.as_mut().unwrap();

        let surface_config = &mut state.surface_config;
        surface_config.height = size.height;
        surface_config.width = size.width;
        state.surface.configure(&state.device, &surface_config);
    }
}
async fn request_adapter(
    instance: &wgpu::Instance,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();
    (adapter, device, queue)
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_title("WIP: Game");
        let window = event_loop.create_window(window_attributes).unwrap();
        self.init(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    _ => {}
                }
            }
            // Ignored events
            WindowEvent::Moved(_) => {}
            WindowEvent::CursorMoved { .. } => {}
            _ => println!("{event:?}"),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
