extern crate sdl2_sys;

use std::os::raw::c_char;

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

    unsafe {
        let _window : *mut sdl2_sys::SDL_Window = sdl2_sys::SDL_CreateWindow(
            "poop".as_ptr() as *const c_char, 0, 0, 800, 600, 0);
    }
}
