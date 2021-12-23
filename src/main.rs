use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Window,
    dpi::PhysicalSize
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle t our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // trace path
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.width,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        return Self {
            surface,
            device,
            queue,
            config,
            size
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config)
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        return false;
    }

    fn update(&mut self) {
        // todo!();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        { // Extra code block to drop the render pass. so encoder can stop being borrowed and finish. Needs better explanation. 
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    }
                }],
                depth_stencil_attachment: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => if !state.input(event) { 
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                        input: 
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so w have to dereference it twice
                    state.resize(**new_inner_size);
                },
                _ => {}
            }
        },
        /*
        * Currently an error right here, where it only tells me its outdated.
        * Need to learn how to debug this error properly, get extra details, which line?, etc.
        * Something in the state.render() perhaps
        */
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {},
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => { eprintln!("Error! {:?}", e); *control_flow = ControlFlow::Exit},
            }
        },
        Event::RedrawEventsCleared => {
            //RedrawRequested will only trigger once, unless we manually request it.
            window.request_redraw();
        }
        _ => {}
    });
}

// match event {
//     // ...

//     } if window_id == window.id() => if !state.input(event) {
//         match event {
//             // ...

//             WindowEvent::Resized(physical_size) => {
//                 state.resize(*physical_size);
//             }
//             WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                 // new_inner_size is &&mut so we have to dereference it twice
//                 state.resize(**new_inner_size);
//             }
//             // ...
// }

