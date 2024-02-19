mod renderer;
mod game;
mod rect;
mod entity;
mod collision_system;
mod gun;
mod kmath;
mod map_gen;
mod map_fragment;
mod priority_queue;

mod level;

use glow::*;
use std::error::Error;
use glam::{Vec3, Mat4};
use kmath::*;
use renderer::*;
use rect::*;
use game::*;
use std::collections::HashSet;
use std::time::{Duration, SystemTime};



fn main() -> Result<(), Box<dyn Error>> {

    let mut window_x = 1600.0;
    let mut window_y = 1200.0;

    let projection_mat = Mat4::orthographic_lh(0.0, 1.0, 1.0, 0.0, 1000.0, 0.0);
    println!("{}", projection_mat);
    let projection_inverse = projection_mat.inverse();


    unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Hello triangle!")
            //.with_inner_size(glutin::dpi::LogicalSize::new(window_x, window_y));
            .with_inner_size(glutin::dpi::PhysicalSize::new(window_x, window_y));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap()
            .make_current()
            .unwrap();
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
        gl.enable(DEPTH_TEST);

        let mut renderer = Renderer::new(&gl, window_x/window_y);

        let program = gl.create_program().expect("Cannot create program");

        {   // Shader stuff
            let shader_version = "#version 410";
            let shader_sources = [
                (glow::VERTEX_SHADER, std::fs::read_to_string("src/test.vert")?),
                (glow::FRAGMENT_SHADER, std::fs::read_to_string("src/test.frag")?),
            ];
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
        }

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        let mut game = Game::new(window_x / window_y);

        let mut held_keys: HashSet<glutin::event::VirtualKeyCode> = HashSet::new();
        let mut lmb = false;
        let mut normalized_cursor_pos = Vec2::new(0.0, 0.0);
        let mut dt = 1.0f64 / 60f64;


        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::LoopDestroyed |
                    Event::WindowEvent {event: WindowEvent::CloseRequested, ..} |
                    Event::WindowEvent {event: WindowEvent::KeyboardInput {
                        input: glutin::event::KeyboardInput { virtual_keycode: Some(glutin::event::VirtualKeyCode::Escape), ..}, ..}, ..}
                    => {
                        gl.delete_program(program);
                        renderer.destroy(&gl);
                        *control_flow = ControlFlow::Exit;
                        return;
                    },


                    Event::MainEventsCleared => {
                        // update
                        let loop_start = SystemTime::now();


                        let mut motion_vec = Vec2::new(0.0, 0.0);
                        if held_keys.contains(&glutin::event::VirtualKeyCode::W) {
                            motion_vec.y -= 1.0;
                        }
                        if held_keys.contains(&glutin::event::VirtualKeyCode::S) {
                            motion_vec.y += 1.0;
                        }
                        if held_keys.contains(&glutin::event::VirtualKeyCode::A) {
                            motion_vec.x -= 1.0;
                        }
                        if held_keys.contains(&glutin::event::VirtualKeyCode::D) {
                            motion_vec.x += 1.0;
                        }
                        if motion_vec != Vec2::new(0.0, 0.0) {
                            game.apply_command(InputCommand::Move(motion_vec.normalize()));
                        } else {
                            game.apply_command(InputCommand::Move(Vec2::new(0.0, 0.0)));
                        }

                        game.update(window_x / window_y, dt as f32);

                        // draw
                        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                        renderer.clear();

                        
                        gl.uniform_matrix_4_f32_slice(gl.get_uniform_location(program, "projection").as_ref(),
                            false, &projection_mat.to_cols_array());
                        
                        game.draw(&mut renderer);

                        renderer.present(&gl);
                        
                        window.swap_buffers().unwrap();

                        /*
                        let loop_end = SystemTime::now();
                        let delta = loop_end.duration_since(loop_start).unwrap().as_secs_f64();
                        let frame_cap = 1.0 / 60.0;
                        // not sure if this handles vsync ay
                        if delta < frame_cap {
                            std::thread::sleep(Duration::from_secs_f64(frame_cap - delta));
                            dt = frame_cap;
                        } else {
                            dt = delta;
                        }
                        */
                    }


                    Event::RedrawRequested(_) => {
                        // idc
                    }


                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            window.resize(*physical_size);
                            window_x = physical_size.width as f32;
                            window_y = physical_size.height as f32;
                            gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                            println!("aspect ratio: {:?}", window_x / window_y);

                        }
                        WindowEvent::CloseRequested => {
                            gl.delete_program(program);
                            renderer.destroy(&gl);
                            *control_flow = ControlFlow::Exit
                        }
                        WindowEvent::KeyboardInput {
                            input: glutin::event::KeyboardInput { virtual_keycode: Some(virtual_code), state, .. },
                            ..
                        } => {
                            match state {
                                glutin::event::ElementState::Pressed => held_keys.insert(*virtual_code),
                                glutin::event::ElementState::Released => held_keys.remove(virtual_code),
                            };

                            match (virtual_code, state) {
                                (glutin::event::VirtualKeyCode::Escape, _) => {
                                    gl.delete_program(program);
                                    renderer.destroy(&gl);
                                    *control_flow = ControlFlow::Exit;
                                },
                                (glutin::event::VirtualKeyCode::R, glutin::event::ElementState::Released) => {
                                    game.apply_command(InputCommand::Reset)
                                },
                            _ => (),
                        }},
                        WindowEvent::MouseInput {
                            button: glutin::event::MouseButton::Right,
                            state: glutin::event::ElementState::Pressed,
                            ..
                        } => {
                            game.apply_command(InputCommand::EatGun);
                        }
                        WindowEvent::MouseInput {
                            button: glutin::event::MouseButton::Left,
                            state:state,
                            ..
                        } => {
                            if *state == glutin::event::ElementState::Pressed {
                                lmb = true;
                                game.apply_command(InputCommand::Shoot(normalized_cursor_pos));
                            } else {
                                lmb = false;
                                game.apply_command(InputCommand::Unshoot);
                            }
                        },
                        WindowEvent::CursorMoved {
                            position: pos,
                            ..
                        } => {
                            normalized_cursor_pos = Vec2::new(
                                pos.x as f32 / window_x * window_x / window_y, 
                                pos.y as f32 / window_y);

                            game.apply_command(InputCommand::Look(normalized_cursor_pos));
                            if lmb {
                                game.apply_command(InputCommand::Shoot(normalized_cursor_pos));
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    }
}