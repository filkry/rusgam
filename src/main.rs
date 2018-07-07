extern crate sdl2_sys;
extern crate winapi;
extern crate wio;

//mod math;
//use math::SVec3f;

mod russdl;
mod rusd3d12;

#[allow(dead_code)]
fn main_sdl() {
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
}

fn main_d3d12() {
    let debuginterface = rusd3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();

    let winapi = rusd3d12::initwinapi().unwrap();
    let windowclass = winapi.registerclassex("rusgam").unwrap();
    let _window = windowclass.createwindow("rusgame2", 800, 600).unwrap();

    let _adapter = rusd3d12::getadapter().unwrap();
}

fn main() {
    //main_sdl();
    main_d3d12();
}
