extern crate sdl2_sys;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::fmt;
use std::mem;

pub struct SErr {
    err: *const c_char,
}

impl fmt::Display for SErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cstr = CStr::from_ptr(self.err).to_str();
            match cstr {
                Ok(errstring) => write!(f, "{}", errstring),
                _ => Err(fmt::Error)
            }
        }
    }
}

impl fmt::Debug for SErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cstr = CStr::from_ptr(self.err).to_str();
            match cstr {
                Ok(errstring) => write!(f, "{}", errstring),
                _ => Err(fmt::Error)
            }
        }
    }
}

unsafe fn lastsdlerr() -> SErr {
    SErr{err: sdl2_sys::SDL_GetError()}
}

pub struct SContext {
}

pub fn init() -> Result<SContext, SErr> {
    unsafe {
        let initres = sdl2_sys::SDL_Init(sdl2_sys::SDL_INIT_VIDEO);
        match initres {
            0 => {
                Ok(SContext{})
            }
            _ => {
                Err(lastsdlerr())
            }
        }
    }
}

impl Drop for SContext {
    fn drop(&mut self) {
        unsafe {
            sdl2_sys::SDL_Quit();
        }
    }
}

#[allow(dead_code)]
pub enum EKeySym {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

pub struct SKeyDownEvent {
    pub symbol: EKeySym,
}

impl SKeyDownEvent {
    pub unsafe fn fromsdlevent(event: sdl2_sys::SDL_Event) -> Option<SKeyDownEvent> {
        let keyboardevent = event.key;
        let keysym = keyboardevent.keysym;

        /*match keysym.sym {
            sdl2_sys::SDLK_q => SKeyDownEvent{symbol: EKeySym::Q},
            _ => None
        }*/

        if keysym.sym == sdl2_sys::SDLK_q as i32 {
            Some(SKeyDownEvent{symbol: EKeySym::Q})
        }
        else {
            None
        }
    }
}

pub enum EEvent {
    KeyDown(SKeyDownEvent),
}

impl EEvent {
    pub unsafe fn fromsdlevent(event: sdl2_sys::SDL_Event) -> Option<EEvent> {
        // -- $$$$FRK(TODO): I'm assuming this if block is inefficient
        // -- $$$$FRK(TODO): unwrap here is loose
        if event.type_ == sdl2_sys::SDL_EventType::SDL_KEYDOWN as u32 {
            let keydownevent = SKeyDownEvent::fromsdlevent(event);
            match keydownevent {
                Some(data) => Some(EEvent::KeyDown(data)),
                None => None
            }
        }
        else {
            None
        }
    }
}

#[allow(dead_code)]
impl SContext {
    pub fn glsetattribute(&self, attribute: sdl2_sys::SDL_GLattr, value: i32) -> Result<(), SErr> {
        unsafe {
            match sdl2_sys::SDL_GL_SetAttribute(attribute, value) {
                0 => Ok(()),
                _ => Err(lastsdlerr())
            }
        }
    }

    pub fn pollevent(&self) -> Option<EEvent> {
        unsafe {
            let mut event = mem::uninitialized();
            if sdl2_sys::SDL_PollEvent(&mut event) == 1 {
                EEvent::fromsdlevent(event)
            }
            else {
                None
            }
        }
    }
}

#[allow(dead_code)]
pub struct SWindow {
    window: *mut sdl2_sys::SDL_Window,
}

#[allow(dead_code)]
impl SContext {
    pub fn createwindow(&self, title: &str, xpos: i32, ypos: i32, width: i32, height: i32) ->Result<SWindow, SErr> {
        // -- $$$FRK(TODO): support flags in this? For now only one use case.

        unsafe {
            let flags = (sdl2_sys::SDL_WindowFlags::SDL_WINDOW_OPENGL as u32) | (sdl2_sys::SDL_WindowFlags::SDL_WINDOW_SHOWN as u32);
            let window = sdl2_sys::SDL_CreateWindow(title.as_ptr() as *const c_char,
                                                    xpos, ypos, width, height, flags);
            if window.is_null() {
                Err(lastsdlerr())
            }
            else {
                Ok(SWindow{window: window})
            }
        }
    }
}

impl Drop for SWindow {
    fn drop(&mut self) {
        unsafe {
            sdl2_sys::SDL_DestroyWindow(self.window);
        }
    }
}

#[allow(dead_code)]
pub struct SGLContext {
    context: sdl2_sys::SDL_GLContext,
}

impl SWindow {
    pub fn createglcontext(&self) -> Result<SGLContext, SErr> {
        unsafe {
            let context = sdl2_sys::SDL_GL_CreateContext(self.window);
            if context.is_null() {
                Err(lastsdlerr())
            }
            else {
                Ok(SGLContext{context: context})
            }
        }
    }
}
