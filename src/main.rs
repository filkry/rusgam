extern crate sdl2_sys;
extern crate winapi;
extern crate wio;

//mod math;
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
    let window = windowclass.createwindow("rusgame2", 800, 600).unwrap();

    let d3d12 = rusd3d12::initd3d12().unwrap();
    let mut adapter = d3d12.getadapter().unwrap();
    let device = adapter.createdevice().unwrap();

    let mut commandqueue = device.createcommandqueue(
        rusd3d12::ECommandListType::Direct).unwrap();
    let swapchain = d3d12.createswapchain(&window, &mut commandqueue, 800, 600).unwrap();

    let rendertargetheap = device.createdescriptorheap(
        rusd3d12::EDescriptorHeapType::RenderTarget,
        10).unwrap();

    device.initrendertargetviews(&swapchain, &rendertargetheap).unwrap();
    let commandallocator = device.createcommandallocator(
        rusd3d12::ECommandListType::Direct).unwrap();
    let _commandlist = device.createcommandlist(&commandallocator).unwrap();

    let _fence = device.createfence().unwrap();

    let mut framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();
    loop {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let dtms = (dt as f64) / 1000.0;

        println!("Frame {} time: {}ms", framecount, dtms);

        lastframetime = curframetime;
        framecount += 1;
    }
}

fn main() {
    //main_sdl();
    main_d3d12();
}
