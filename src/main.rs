extern crate sdl2_sys;

mod math;
use math::SVec3f;

mod russdl;

//use std::os::raw::c_char;

struct STestStruct {
    x : u64,
    blech : String,
    y : i32,
}

fn main() {
    let x : u64 = 64;
    let teststruct = STestStruct {
        x : 24,
        blech : "poopsock".to_string(),
        y : -5,
    };
    println!("Hello, world {}!", x);
    println!("Teststruct: {}, {}, {}", teststruct.x, teststruct.blech, teststruct.y);

    let sdlcontext = russdl::init().unwrap();
    sdlcontext.glsetattribute(sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK,
                              sdl2_sys::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as i32).unwrap();
    sdlcontext.glsetattribute(sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_MAJOR_VERSION, 3).unwrap();
    sdlcontext.glsetattribute(sdl2_sys::SDL_GLattr::SDL_GL_CONTEXT_MINOR_VERSION, 3).unwrap();

    let window = sdlcontext.createwindow("rusgam", 30, 30, 800, 600).unwrap();
    let _context = window.createglcontext().unwrap();

    let mut quit = false;
    while !quit {
        loop {
            match sdlcontext.pollevent() {
                Some(event) => {
                    match event {
                        russdl::EEvent::KeyDown(keydownevent) => {
                            match keydownevent.symbol {
                                russdl::EKeySym::Q => {
                                    quit = true;
                                },
                                _ => () // unused keys
                            }
                        },
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    let mut myvec = SVec3f::default();
    myvec.y = -2.0;
    println!("Vec: {}", 1.3 * myvec);
}
