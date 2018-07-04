extern crate sdl2_sys;

mod math;
use math::SVec3f;

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
            "rusgam".as_ptr() as *const c_char, 0, 0, 800, 600, 0);
    }

    let rad = (23.4 as f32).to_radians();
    println!("Deg to rad {}", rad);

    let mut myvec = SVec3f::default();
    myvec.y = -2.0;
    println!("Vec: {}", 1.3 * myvec);
}
