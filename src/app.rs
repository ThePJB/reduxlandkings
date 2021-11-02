use glow::*;
use std::error::Error;
use crate::renderer::*;
use crate::rect::*;
use glam::*;

struct App {
    renderer: Renderer,

    gl: glow::Context,
    event_loop: glutin::event_loop::EventLoop<()>,
    window: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    program: glow::NativeProgram,
}

impl App {
    pub fn new() -> Result<App, Box<dyn Error>> {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Hello triangle!")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap()
            .make_current()
            .unwrap();
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
        let program = gl.create_program().expect("Cannot create program");

        let mut renderer = Renderer::new(&gl);

        let shader_sources = [
            (glow::VERTEX_SHADER, std::fs::read_to_string("src/test.vert")?),
            (glow::FRAGMENT_SHADER, std::fs::read_to_string("src/test.frag")?),
        ];

        let shader_version = "#version 410";

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);

        Ok(App{gl, renderer, window, event_loop, program})
    }

    pub fn run(&mut self) {
        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::LoopDestroyed => {
                    return;
                }
                Event::MainEventsCleared => {
                    self.window.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    self.gl.clear(glow::COLOR_BUFFER_BIT);

                    self.renderer.clear();
                    
                    self.renderer.draw_rect(Rect::new(0.0, 0.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0), 1.0);
                    self.renderer.draw_rect(Rect::new(0.2, 0.2, 0.5, 0.2), Vec3::new(0.0, 0.0, 1.0), 1.0);
                    self.renderer.draw_rect(Rect::new(0.6, 0.8, 0.2, 0.3), Vec3::new(0.0, 1.0, 0.0), 1.0);

                    self.renderer.present(&self.gl);
                    
                    self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
                    self.window.swap_buffers().unwrap();
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        self.window.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        self.gl.delete_program(self.program);
                        self.renderer.destroy(&self.gl);
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::KeyboardInput {
                        input: glutin::event::KeyboardInput { virtual_keycode: Some(virtual_code), state, .. },
                        ..
                    } => match (virtual_code, state) {
                        (glutin::event::VirtualKeyCode::Escape, _) => {
                            self.gl.delete_program(self.program);
                            self.renderer.destroy(&self.gl);
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        });
    }
}