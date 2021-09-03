#![allow(dead_code)]
use std::{cmp, fmt, mem, ptr};
use std::convert::TryFrom;

use crate::win;

pub mod rawinput;

#[macro_export]
macro_rules! break_err {
    ($e:expr) => {
        safewindows::break_if_debugging();
        return $e;
    };
}

#[macro_export]
macro_rules! break_assert {
    ($e:expr) => {
        if !($e) {
            safewindows::break_if_debugging();
            assert!(false);
        }
    };
}

pub struct SErr {
    errcode: win::WIN32_ERROR,
}

pub unsafe fn getlasterror() -> SErr {
    SErr {
        errcode: win::GetLastError(),
    }
}

impl fmt::Debug for SErr {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        // -- $$$FRK(TODO): we can call GetLastError to impl Debug/Display for SErr
        Ok(())
    }
}

pub struct SWinAPI {
    hinstance: win::HINSTANCE,
}

pub fn initwinapi() -> Result<SWinAPI, SErr> {
    unsafe {
        let hinstance = win::GetModuleHandleW(None);
        if !hinstance.is_null() {
            Ok(SWinAPI {
                hinstance: hinstance,
            })
        } else {
            Err(getlasterror())
        }
    }
}

pub struct SWindowClass<'windows> {
    winapi: &'windows SWinAPI,
    windowclassname: &'static str,
    class: u16,
}

impl<'windows> Drop for SWindowClass<'windows> {
    fn drop(&mut self) {
        unsafe {
            win::UnregisterClassW(
                self.windowclassname,
                self.winapi.hinstance,
            );
        }
    }
}

unsafe extern "system" fn windowproctrampoline(
    hwnd: win::HWND,
    msg: u32,
    wparam: win::WPARAM,
    lparam: win::LPARAM,
) -> win::LRESULT {
    let window_ptr = win::GetWindowLongPtrW(hwnd, win::GWLP_USERDATA) as *mut SWindow;
    if !window_ptr.is_null() {
        assert!(hwnd == (*window_ptr).window);
        return (*window_ptr).windowproc(msg, wparam, lparam);
    }
    win::DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub struct SEventHandle {
    event: win::HANDLE,
}

impl SEventHandle {
    pub unsafe fn raw(&self) -> win::HANDLE {
        self.event
    }

    pub fn waitforsingleobject(&self, duration: u32) {
        unsafe { win::WaitForSingleObject(self.raw(), duration) };
    }
}

impl SWinAPI {
    pub fn queryperformancecounter() -> i64 {
        let mut result : i64 = 0;
        let success = unsafe { win::QueryPerformanceCounter(&mut result) };
        if !success.as_bool() {
            panic!("Can't query performance.");
        }

        result
    }

    pub unsafe fn queryperformancefrequencycounter() -> i64 {
        let mut result : i64 = 0;
        let success = win::QueryPerformanceFrequency(&mut result);
        if !success.as_bool() {
            panic!("Can't query performance.");
        }

        result
    }

    fn register_class_ex_internal(&self, windowclassname: win::PWSTR) -> Result<u16, SErr> {
        unsafe {
            let classdata = win::WNDCLASSEXW {
                cbSize: mem::size_of::<win::WNDCLASSEXW>() as u32,
                style: win::CS_HREDRAW | win::CS_VREDRAW,
                lpfnWndProc: Some(windowproctrampoline), //wndproc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: self.hinstance,
                hIcon: win::LoadIconW(self.hinstance, ""),
                hCursor: win::LoadCursorW(self.hinstance, win::IDC_ARROW),
                //hbrBackground: (win::COLOR_WINDOW + 1) as win::HBRUSH,
                //lpszClassName: windowclassname,
                lpszClassName: windowclassname,
                ..Default::default()
            };

            let atom = win::RegisterClassExW(&classdata);
            if atom > 0 {
                Ok(atom)
            } else {
                Err(getlasterror())
            }
        }
    }

    pub fn registerclassex(&self, windowclassname: &'static str) -> Result<SWindowClass, SErr> {
        use windows::IntoParam;
        let mut conversion : win::Param<win::PWSTR> = windowclassname.into_param();
        let atom = self.register_class_ex_internal(conversion.abi())?;

        Ok(SWindowClass {
            winapi: self,
            windowclassname: windowclassname,
            class: atom,
        })
    }

    pub fn createeventhandle(&self) -> Result<SEventHandle, &'static str> {
        let event = unsafe { win::CreateEventW(ptr::null_mut(), false, false, "") };

        if event.is_null() {
            return Err("Couldn't create event.");
        }

        Ok(SEventHandle { event: event })
    }

    pub fn get_cursor_pos(&self) -> [u32; 2] {
        let mut point = win::POINT {
            x: 0,
            y: 0,
        };
        let success = unsafe { win::GetCursorPos(&mut point) };
        assert!(success.as_bool());

        [
            point.x as u32,
            point.y as u32,
        ]
    }
}

pub struct SWindow {
    window: win::HWND,
    windowproc: Option<*mut dyn TWindowProc>,

    registereduserdata: bool,
}

pub struct SMSG {
    msg: win::MSG,
}

impl SWindow {
    pub unsafe fn raw(&self) -> win::HWND {
        self.window
    }

    pub fn show(&mut self) {
        unsafe { win::ShowWindow(self.window, win::SW_SHOW) };
    }

    pub fn beginpaint(&mut self) {
        unsafe {
            // -- $$$FRK(TODO): real paintstruct
            let mut paintstruct = mem::MaybeUninit::<win::PAINTSTRUCT>::uninit();
            win::BeginPaint(self.window, paintstruct.as_mut_ptr());
        }
    }

    pub fn endpaint(&mut self) {
        unsafe {
            // -- $$$FRK(TODO): real paintstruct
            let mut paintstruct = mem::MaybeUninit::<win::PAINTSTRUCT>::uninit();
            win::EndPaint(self.window, paintstruct.as_mut_ptr());
        }
    }

    // -- calls the stored windowproc if it exists, otherwise uses a default windowproc
    pub unsafe fn windowproc(&mut self, msg: u32, wparam: win::WPARAM, lparam: win::LPARAM) -> win::LRESULT {
        match self.windowproc {
            Some(mptr) => match mptr.as_mut() {
                Some(m) => {
                    let msgtype: EMsgType = msgtype(msg, wparam, lparam);
                    m.windowproc(self, msgtype);
                    win::DefWindowProcW(self.window, msg, wparam, lparam)
                }
                None => win::DefWindowProcW(self.window, msg, wparam, lparam),
            },
            None => win::DefWindowProcW(self.window, msg, wparam, lparam),
        }
    }

    // -- registers user data needed for windowproc
    unsafe fn registeruserdata(&mut self) {
        let outwindowptr = self as *mut SWindow as isize;
        win::SetWindowLongPtrW(self.window, win::GWLP_USERDATA, outwindowptr);
    }

    unsafe fn setwindowproc<'a>(&mut self, windowproc: &'a mut dyn TWindowProc) {
        let staticlifetimeptr = std::mem::transmute::<
            &'a mut dyn TWindowProc,
            &'static mut dyn TWindowProc,
        >(windowproc);

        self.windowproc = Some(staticlifetimeptr);
    }

    fn clearwindowproc(&mut self) {
        self.windowproc = None;
    }

    pub fn peekmessage<'a>(&mut self, windowproc: &'a mut dyn TWindowProc) -> Option<SMSG> {
        unsafe {
            if !self.registereduserdata {
                self.registeruserdata();
            }

            let mut raw_msg = mem::MaybeUninit::<win::MSG>::zeroed();

            self.setwindowproc(windowproc);
            let foundmessage = win::PeekMessageW(
                raw_msg.as_mut_ptr(),
                self.window,
                0,
                0,
                win::PM_REMOVE,
            );
            self.clearwindowproc();

            if foundmessage.as_bool() {
                Some(SMSG {
                    msg: raw_msg.assume_init(),
                })
            } else {
                None
            }
        }
    }

    pub fn translatemessage<'a>(&mut self, message: &mut SMSG) {
        unsafe {
            win::TranslateMessage(&mut message.msg);
        }
    }

    pub fn dispatchmessage<'a>(&mut self, message: &mut SMSG, windowproc: &'a mut dyn TWindowProc) {
        unsafe {
            if !self.registereduserdata {
                self.registeruserdata();
            }

            self.setwindowproc(windowproc);
            win::DispatchMessageW(&mut message.msg);
            self.clearwindowproc();
        }
    }

    pub fn getclientrect(&self) -> Result<SRect, &'static str> {
        unsafe {
            let mut rect: win::RECT = mem::zeroed();
            let res = win::GetClientRect(self.window, &mut rect as *mut win::RECT);
            if !res.as_bool() {
                return Err("Could not get client rect.");
            }

            Ok(SRect {
                left: rect.left,
                right: rect.right,
                top: rect.top,
                bottom: rect.bottom,
            })
        }
    }

    pub fn screen_to_client(&self, point: &[u32; 2]) -> [i32; 2] {
        let mut point = win::POINT {
            x: point[0] as i32,
            y: point[1] as i32,
        };

        let success = unsafe { win::ScreenToClient(self.window, &mut point) };
        assert!(success.as_bool());

        [point.x, point.y]
    }
}

#[derive(Copy, Clone)]
pub struct SRect {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

pub trait TWindowProc {
    fn windowproc(&mut self, window: &mut SWindow, msg: EMsgType) -> ();
}

impl<'windows> SWindowClass<'windows> {
    pub fn createwindow(&self, title: &str, width: u32, height: u32) -> Result<SWindow, SErr> {
        unsafe {
            let windowstyle = win::WS_OVERLAPPEDWINDOW;

            let screenwidth = win::GetSystemMetrics(win::SM_CXSCREEN);
            let screenheight = win::GetSystemMetrics(win::SM_CYSCREEN);

            let mut windowrect = win::RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            win::AdjustWindowRect(&mut windowrect, windowstyle, false);

            let windowwidth = windowrect.right - windowrect.left;
            let windowheight = windowrect.bottom - windowrect.top;

            let windowx = cmp::max(0, (screenwidth - windowwidth) / 2);
            let windowy = cmp::max(0, (screenheight - windowheight) / 2);

            //self.class as ntdef::LPCWSTR,
            let hinstanceparam = self.winapi.hinstance;

            let hwnd: win::HWND = win::CreateWindowExW(
                win::WINDOW_EX_STYLE::default(),
                self.windowclassname,
                title,
                windowstyle,
                windowx,
                windowy,
                windowwidth,
                windowheight,
                win::HWND::NULL,
                win::HMENU::NULL,
                hinstanceparam,
                ptr::null_mut(),
            );

            if !hwnd.is_null() {
                Ok(SWindow {
                    window: hwnd,
                    registereduserdata: false,
                    windowproc: None,
                })
            } else {
                Err(getlasterror())
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum EKey {
    Invalid,
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
    Number1,
    Number2,
    Number3,
    Number4,
    Number5,
    Number6,
    Number7,
    Number8,
    Number9,
    Number0,
    Space,
    Enter,
    Escape,
    KeyPadEnter,
    Backspace,
    Tilde,
    Tab,
    LeftArrow,
    RightArrow,
    DownArrow,
    UpArrow,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Delete,
    Minus,
}

pub fn translatewmkey(key: win::WPARAM) -> EKey {
    match key.0 as u32 {
        0x20 => EKey::Space,
        0x41 => EKey::A,
        0x42 => EKey::B,
        0x43 => EKey::C,
        0x44 => EKey::D,
        0x45 => EKey::E,
        0x46 => EKey::F,
        0x47 => EKey::G,
        0x48 => EKey::H,
        0x49 => EKey::I,
        0x4A => EKey::J,
        0x4B => EKey::K,
        0x4C => EKey::L,
        0x4D => EKey::M,
        0x4E => EKey::N,
        0x4F => EKey::O,
        0x50 => EKey::P,
        0x51 => EKey::Q,
        0x52 => EKey::R,
        0x53 => EKey::S,
        0x54 => EKey::T,
        0x55 => EKey::U,
        0x56 => EKey::V,
        0x57 => EKey::W,
        0x58 => EKey::X,
        0x59 => EKey::Y,
        0x5A => EKey::Z,
        0x30 => EKey::Number0,
        0x31 => EKey::Number1,
        0x32 => EKey::Number2,
        0x33 => EKey::Number3,
        0x34 => EKey::Number4,
        0x35 => EKey::Number5,
        0x36 => EKey::Number6,
        0x37 => EKey::Number7,
        0x38 => EKey::Number8,
        0x39 => EKey::Number9,
        win::VK_OEM_3 => EKey::Tilde,
        win::VK_TAB => EKey::Tab,
        win::VK_LEFT => EKey::LeftArrow,
        win::VK_RIGHT => EKey::RightArrow,
        win::VK_UP => EKey::UpArrow,
        win::VK_DOWN => EKey::DownArrow,
        win::VK_PRIOR => EKey::PageUp,
        win::VK_NEXT => EKey::PageDown,
        win::VK_HOME => EKey::Home,
        win::VK_END => EKey::End,
        win::VK_INSERT => EKey::Insert,
        win::VK_DELETE => EKey::Delete,
        win::VK_BACK => EKey::Backspace,
        win::VK_RETURN => EKey::Enter,
        win::VK_ESCAPE => EKey::Escape,
        win::VK_OEM_MINUS => EKey::Minus,
        _ => EKey::Invalid,
    }
}

#[derive(Copy, Clone)]
pub enum EMsgType {
    Invalid,
    KeyDown { key: EKey },
    KeyUp { key: EKey },
    LButtonDown { x_pos: i32, y_pos: i32 },
    LButtonUp { x_pos: i32, y_pos: i32 },
    MButtonDown { x_pos: i32, y_pos: i32 },
    MButtonUp { x_pos: i32, y_pos: i32 },
    Paint,
    Size,
    Input { raw_input: rawinput::SRawInput },
}

pub fn msgtype(msg: u32, wparam: win::WPARAM, lparam: win::LPARAM) -> EMsgType {
    match msg {
        win::WM_KEYDOWN => EMsgType::KeyDown {
            key: translatewmkey(wparam),
        },
        win::WM_KEYUP => EMsgType::KeyUp {
            key: translatewmkey(wparam),
        },
        win::WM_LBUTTONDOWN => EMsgType::LButtonDown {
            x_pos: win::GET_X_LPARAM(lparam),
            y_pos: win::GET_Y_LPARAM(lparam),
        },
        win::WM_LBUTTONUP => EMsgType::LButtonUp {
            x_pos: win::GET_X_LPARAM(lparam),
            y_pos: win::GET_Y_LPARAM(lparam),
        },
        win::WM_MBUTTONDOWN => EMsgType::MButtonDown {
            x_pos: win::GET_X_LPARAM(lparam),
            y_pos: win::GET_Y_LPARAM(lparam),
        },
        win::WM_MBUTTONUP => EMsgType::MButtonUp {
            x_pos: win::GET_X_LPARAM(lparam),
            y_pos: win::GET_Y_LPARAM(lparam),
        },
        win::WM_PAINT => EMsgType::Paint,
        win::WM_SIZE => EMsgType::Size,
        win::WM_INPUT => {
            const RI_SIZE : u32 = std::mem::size_of::<win::RAWINPUT>() as u32;
            const RI_HEADER_SIZE : u32 = std::mem::size_of::<win::RAWINPUTHEADER>() as u32;

            let mut bytes : [u8; RI_SIZE as usize] = [0; RI_SIZE as usize];

            unsafe {

                #[allow(const_item_mutation)]
                let result = win::GetRawInputData(
                    win::HRAWINPUT(lparam.0),
                    win::RID_INPUT,
                    &mut bytes[0] as *mut u8 as *mut std::ffi::c_void,
                    &mut RI_SIZE,
                    RI_HEADER_SIZE
                );

                if result == std::u32::MAX {
                    panic!("Bad message.");
                }

                let raw : *mut win::RAWINPUT = &mut bytes[0] as *mut u8 as *mut win::RAWINPUT;

                let raw_input = rawinput::SRawInput::try_from(*raw).unwrap();

                EMsgType::Input { raw_input }

            }
        },
        _ => EMsgType::Invalid,
    }
}

pub fn debug_break() {
    unsafe { win::DebugBreak() };
}

pub fn break_if_debugging() {
    if unsafe { win::IsDebuggerPresent().as_bool() } {
        debug_break();
    }
}
