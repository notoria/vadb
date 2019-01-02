extern crate gl;
extern crate glutin;
extern crate png;

use gl::types::*;
use glutin::GlContext;
use std::ffi::CString;
use std::fs::File;
use std::process::Command;
use std::str;
use std::thread;
use std::time::{Duration, Instant};
use std::{mem, ptr};

macro_rules! check_error {
    //($s: expr)	=> { unsafe {
    ($s:expr) => {{
        let err = gl::GetError();
        match err {
            gl::NO_ERROR => (),
            gl::INVALID_ENUM => panic!("enum: {}", $s),
            gl::INVALID_VALUE => panic!("value: {}", $s),
            gl::INVALID_OPERATION => panic!("operation: {}", $s),
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                panic!("framebuffer operation: {}", $s)
            }
            gl::OUT_OF_MEMORY => panic!("out of memory: {}", $s),
            gl::STACK_UNDERFLOW => panic!("stack underflow: {}", $s),
            gl::STACK_OVERFLOW => panic!("stack overflow: {}", $s),
            _ => panic!("Unknow: {}", $s),
        }
        if err != gl::NO_ERROR {
            panic!("Error: {}", $s);
        }
    }};
}

static SCREEN_PATH: &str = "/tmp/screen.png";

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        // Fail on error
        if status != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!("{}", str::from_utf8(&buf).unwrap());
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        // Fail on error
        if status != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!("{}", str::from_utf8(&buf).unwrap());
        }
    }
    program
}

fn read_shader_files(vertex: &str, fragment: &str) -> (Vec<u8>, Vec<u8>) {
    use std::fs::File;
    use std::io::Read;
    let mut file_vertex = File::open(vertex).unwrap();
    let mut file_fragment = File::open(fragment).unwrap();
    let mut vec_vertex = Vec::new();
    let mut vec_fragment = Vec::new();
    file_vertex.read_to_end(&mut vec_vertex).unwrap();
    file_fragment.read_to_end(&mut vec_fragment).unwrap();
    (vec_vertex, vec_fragment)
}

fn load_texture(texture: GLuint, texture_path: &str) -> (u32, u32) {
    unsafe {
        let (x, y);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        if texture_path != "" {
            let decoder = png::Decoder::new(File::open(texture_path).unwrap());
            let (info, mut reader) = decoder.read_info().unwrap();
            //(x, y) = (info.width, info.height);
            x = info.width;
            y = info.height;
            let mut buf = vec![0; info.buffer_size()];
            //reader.next_frame(&mut buf).unwrap();
            reader.next_frame(&mut buf);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as _,
                info.width as _,
                info.height as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as _
                //mem::transmute(buf.as_ptr()),
            );
        } else {
            panic!("Can't open file");
        }
        gl::BindTexture(gl::TEXTURE_2D, 0);
        (x, y)
    }
}

fn run_commands() {
    let _ = Command::new("adb")
        .arg("shell")
        .arg("screencap")
        .arg("-p")
        .arg("/sdcard/screen.png")
        //.spawn()
        .status()
        .expect("failed to execute process");
    let _ = Command::new("adb")
        .arg("pull")
        .arg("/sdcard/screen.png")
        .arg(SCREEN_PATH)
        //.spawn()
        //.output()
        .status()
        .expect("failed to execute process");
    //println!("status: {}", output.status);
    thread::sleep(Duration::from_millis(100));
}

fn tap(x: u32, y: u32) {
    println!("x {:.0} y {:.0}", x, y);
    let _ = Command::new("adb")
        .arg("shell")
        .arg("input")
        .arg("tap")
        .arg(format!("{:.0}", x))
        .arg(format!("{:.0}", y))
        .status()
        .expect("failed to execute process");
}

fn swipe(x0: u32, y0: u32, x1: u32, y1: u32, ms: u32) {
    println!(
        "x0 {:.0} y0 {:.0} x1 {:.0} y1 {:.0} t {}",
        x0, y0, x1, y1, ms
    );
    let _ = Command::new("adb")
        .arg("shell")
        .arg("input")
        .arg("swipe")
        .arg(format!("{:.0}", x0))
        .arg(format!("{:.0}", y0))
        .arg(format!("{:.0}", x1))
        .arg(format!("{:.0}", y1))
        .arg(format!("{:.0}", ms))
        .status()
        .expect("failed to execute process");
}

fn event_code(code: usize) {
    let _ = Command::new("adb")
        .arg("shell")
        .arg("input")
        .arg("keyevent")
        .arg(format!("{}", code))
        .status()
        .expect("failed to execute process");
}

fn main() {
    let mut swipe_timer = Instant::now();
    let vertices: Vec<f32> = vec![
        -1., -1., 1., -1., 1., 1., // first triangle
        -1., -1., 1., 1., -1., 1., // seconde triangle
    ];
    let (mut width, mut height) = (256, 64);
    let mut events_loop = glutin::EventsLoop::new();
    let (mut mouse_x, mut mouse_y) = (0., 0.);
    let (mut s_x, mut s_y) = (0., 0.);
    let mut reload = true;
    let mut running = true;
    let mut texture = 0;
    let mut vao = 0;
    let mut vbo = 0;
    let alpha = 0.385;
    let program;
    let window = glutin::WindowBuilder::new()
        .with_title("vadb")
        .with_dimensions(width, height);
    let context = glutin::ContextBuilder::new(); //.with_vsync(true);
    let gl_window =
        glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    unsafe {
        let (vs_src, fs_src) =
            read_shader_files("shaders/main.vert", "shaders/main.frag");
        let vs =
            compile_shader(str::from_utf8(&vs_src).unwrap(), gl::VERTEX_SHADER);
        let fs = compile_shader(
            str::from_utf8(&fs_src).unwrap(),
            gl::FRAGMENT_SHADER,
        );
        program = link_program(vs, fs);
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        // Create texture
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_BORDER as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_BORDER as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            //	gl::LINEAR as _);
            gl::NEAREST as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            //	gl::LINEAR as _);
            gl::NEAREST as _,
        );
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as _,
            //mem::transmute(vertices.as_ptr()),
            gl::STATIC_DRAW,
        );
        //gl::BindVertexArray(0);
        check_error!("vao");
        gl::UseProgram(program);
        let target = CString::new("target").unwrap();
        gl::BindFragDataLocation(
            program,
            0,
            target.as_ptr(),
        );
        //*
        let str_u_pos = CString::new("u_pos").unwrap();
        let u_pos = gl::GetAttribLocation(
            program,
            str_u_pos.as_ptr(),
        );
        gl::EnableVertexAttribArray(u_pos as GLuint);
        gl::VertexAttribPointer(
            u_pos as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        let str_tex0 = CString::new("tex0").unwrap();
        let tex0 = gl::GetUniformLocation(
            program,
            str_tex0.as_ptr(),
        );
        gl::Uniform1i(tex0, 0);
        // */
        check_error!("end");
        gl::BindVertexArray(0);
    }
    while running {
        unsafe {
            if reload {
                run_commands();
                let (mut w, mut h) = load_texture(texture, SCREEN_PATH);
                w = (alpha * w as f32) as _;
                h = (alpha * h as f32) as _;
                width = w;
                height = h;
                gl_window.set_inner_size(w, h);
                reload = false;
            }
            events_loop.poll_events(|event| {
                //events_loop.run_forever(|event| {
                use glutin::WindowEvent;
                use glutin::{
                    ElementState, KeyboardInput, MouseButton, VirtualKeyCode,
                };
                match event {
                    glutin::Event::WindowEvent { event, .. } => {
                        // use glutin::Event;
                        match event {
                            WindowEvent::Closed => running = false,
                            WindowEvent::MouseMoved {
                                position: (x, y),
                                ..
                            } => {
                                mouse_x = x;
                                mouse_y = y;
                            }
                            WindowEvent::MouseInput {
                                state: ElementState::Pressed,
                                button: MouseButton::Left,
                                ..
                            } => {
                                s_x = mouse_x;
                                s_y = mouse_y;
                                swipe_timer = Instant::now();
                            }
                            WindowEvent::MouseInput {
                                state: ElementState::Released,
                                button: MouseButton::Left,
                                ..
                            } => {
                                let distance = (s_x - mouse_x).powf(2.)
                                    + (s_y - mouse_y).powf(2.);
                                if distance < 1.0 {
                                    tap(
                                        (mouse_x as f32 / alpha) as u32,
                                        (mouse_y as f32 / alpha) as u32,
                                    );
                                } else {
                                    let t = (swipe_timer.elapsed().as_secs()
                                        * 1000)
                                        as u32
                                        + (swipe_timer.elapsed().subsec_nanos()
                                            / 1_000_000)
                                            as u32;
                                    swipe(
                                        (s_x as f32 / alpha) as u32,
                                        (s_y as f32 / alpha) as u32,
                                        (mouse_x as f32 / alpha) as u32,
                                        (mouse_y as f32 / alpha) as u32,
                                        t,
                                    );
                                }
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::LControl),
                                        ..
                                    },
                                ..
                            } => reload = true,
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(4); // KEYCODE_BACK
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::F1),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(82); // KEYCODE_MENU
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::Up),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(19); // KEYCODE_DPAD_UP
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::Down),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(20); // KEYCODE_DPAD_DOWN
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::Left),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(21); // KEYCODE_DPAD_LEFT
                                reload = true;
                            }
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode:
                                            Some(VirtualKeyCode::Right),
                                        ..
                                    },
                                ..
                            } => {
                                event_code(22); // KEYCODE_DPAD_RIGHT
                                reload = true;
                            }
                            s => println!("event: {:?}", s),
                            // _ => (),
                        }
                    }
                    _ => (),
                }
                //glutin::ControlFlow::Break
            });
            gl::Viewport(0, 0, width as _, height as _);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0., 0., 0., 1.);
            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            let str_tex0 = CString::new("tex0").unwrap();
            let tex0 = gl::GetUniformLocation(
                program,
                str_tex0.as_ptr(),
            );
            gl::Uniform1i(tex0, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::Flush();
            gl::Finish();
            gl::BindVertexArray(0);
            gl::UseProgram(0);
            gl_window.swap_buffers().unwrap();
        }
    }
}
