#![allow(dead_code)]

use std::{cmp, fmt, mem, ptr};
use std::convert::TryFrom;

// -- $$$FRK(TODO): I feel very slightly guilty about all these wildcard uses
use winapi::shared::basetsd::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef;
use winapi::shared::windef::*;
use winapi::um::winnt::LONG;
use winapi::um::winuser::*;
use winapi::um::{errhandlingapi, libloaderapi, profileapi, synchapi, unknwnbase, winnt};
use winapi::Interface;

use wio::com::ComPtr;

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

// -- this is copied in safeD3D12, does it have to be?
trait ComPtrPtrs<T> {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T>
where
    T: Interface,
{
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown {
        self.as_raw() as *mut unknwnbase::IUnknown
    }
}

// -- $$$FRK(TODO): need to decide what I'm doing with errors re: HRESULT and DWORD errcodes -
// maybe a union?
pub struct SErr {
    errcode: DWORD,
}

pub unsafe fn getlasterror() -> SErr {
    SErr {
        errcode: errhandlingapi::GetLastError(),
    }
}

impl fmt::Debug for SErr {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        // -- $$$FRK(TODO): we can call GetLastError to impl Debug/Display for SErr
        Ok(())
    }
}

pub struct SWinAPI {
    hinstance: HINSTANCE,
}

pub fn initwinapi() -> Result<SWinAPI, SErr> {
    unsafe {
        let hinstance = libloaderapi::GetModuleHandleW(ntdef::NULL as *const u16);
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
    class: ATOM,
}

impl<'windows> Drop for SWindowClass<'windows> {
    fn drop(&mut self) {
        unsafe {
            winapi::um::winuser::UnregisterClassW(
                self.windowclassname.as_ptr() as *const winnt::WCHAR,
                self.winapi.hinstance,
            );
        }
    }
}

unsafe extern "system" fn windowproctrampoline(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SWindow;
    if !window_ptr.is_null() {
        assert!(hwnd == (*window_ptr).window);
        return (*window_ptr).windowproc(msg, wparam, lparam);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub struct SEventHandle {
    event: winnt::HANDLE,
}

impl SEventHandle {
    pub unsafe fn raw(&self) -> winnt::HANDLE {
        self.event
    }

    pub fn waitforsingleobject(&self, duration: u64) {
        unsafe { synchapi::WaitForSingleObject(self.raw(), duration as DWORD) };
    }
}

impl SWinAPI {
    pub fn queryperformancecounter() -> i64 {
        let mut result = mem::MaybeUninit::<winnt::LARGE_INTEGER>::zeroed();
        let success = unsafe { profileapi::QueryPerformanceCounter(result.as_mut_ptr()) };
        if success == 0 {
            panic!("Can't query performance.");
        }

        unsafe { *result.assume_init().QuadPart() }
    }

    pub unsafe fn queryperformancefrequencycounter() -> i64 {
        let mut result = mem::MaybeUninit::<winnt::LARGE_INTEGER>::zeroed();
        let success = profileapi::QueryPerformanceFrequency(result.as_mut_ptr());
        if success == 0 {
            panic!("Can't query performance.");
        }

        *result.assume_init().QuadPart()
    }

    pub fn registerclassex(&self, windowclassname: &'static str) -> Result<SWindowClass, SErr> {
        unsafe {
            let classdata = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(windowproctrampoline), //wndproc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: self.hinstance,
                hIcon: LoadIconW(self.hinstance, ntdef::NULL as *const u16),
                hCursor: LoadCursorW(ntdef::NULL as HINSTANCE, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: ntdef::NULL as *const u16,
                lpszClassName: windowclassname.as_ptr() as *const winnt::WCHAR,
                hIconSm: ntdef::NULL as HICON,
            };

            let atom = RegisterClassExW(&classdata);
            if atom > 0 {
                Ok(SWindowClass {
                    winapi: self,
                    windowclassname: windowclassname,
                    class: atom,
                })
            } else {
                Err(getlasterror())
            }
        }
    }

    pub fn createeventhandle(&self) -> Result<SEventHandle, &'static str> {
        let event = unsafe { synchapi::CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null()) };

        if event == ntdef::NULL {
            return Err("Couldn't create event.");
        }

        Ok(SEventHandle { event: event })
    }

    pub fn get_cursor_pos(&self) -> [u32; 2] {
        let mut point = winapi::shared::windef::POINT {
            x: 0,
            y: 0,
        };
        let success = unsafe { winapi::um::winuser::GetCursorPos(&mut point) };
        assert!(success != 0);

        [
            point.x as u32,
            point.y as u32,
        ]
    }
}

pub struct SWindow {
    window: HWND,
    windowproc: Option<*mut dyn TWindowProc>,

    registereduserdata: bool,
}

pub struct SMSG {
    msg: winapi::um::winuser::MSG,
}

impl SWindow {
    pub unsafe fn raw(&self) -> HWND {
        self.window
    }

    pub fn show(&mut self) {
        unsafe { ShowWindow(self.window, SW_SHOW) };
    }

    pub fn beginpaint(&mut self) {
        unsafe {
            // -- $$$FRK(TODO): real paintstruct
            let mut paintstruct = mem::MaybeUninit::<winapi::um::winuser::PAINTSTRUCT>::uninit();
            winapi::um::winuser::BeginPaint(self.window, paintstruct.as_mut_ptr());
        }
    }

    pub fn endpaint(&mut self) {
        unsafe {
            // -- $$$FRK(TODO): real paintstruct
            let mut paintstruct = mem::MaybeUninit::<winapi::um::winuser::PAINTSTRUCT>::uninit();
            winapi::um::winuser::EndPaint(self.window, paintstruct.as_mut_ptr());
        }
    }

    // -- calls the stored windowproc if it exists, otherwise uses a default windowproc
    pub unsafe fn windowproc(&mut self, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match self.windowproc {
            Some(mptr) => match mptr.as_mut() {
                Some(m) => {
                    let msgtype: EMsgType = msgtype(msg, wparam, lparam);
                    m.windowproc(self, msgtype);
                    DefWindowProcW(self.window, msg, wparam, lparam)
                }
                None => DefWindowProcW(self.window, msg, wparam, lparam),
            },
            None => DefWindowProcW(self.window, msg, wparam, lparam),
        }
    }

    // -- registers user data needed for windowproc
    unsafe fn registeruserdata(&mut self) {
        let outwindowptr = self as *mut SWindow as LONG_PTR;
        SetWindowLongPtrW(self.window, GWLP_USERDATA, outwindowptr);
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

            let mut raw_msg = mem::MaybeUninit::<winapi::um::winuser::MSG>::zeroed();

            self.setwindowproc(windowproc);
            // -- $$$FRK(TODO): this can take a lot more options, but we're hardcoding for now
            let foundmessage = winapi::um::winuser::PeekMessageW(
                raw_msg.as_mut_ptr(),
                self.window,
                0,
                0,
                winapi::um::winuser::PM_REMOVE,
            );
            self.clearwindowproc();

            if foundmessage > 0 {
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
            winapi::um::winuser::TranslateMessage(&mut message.msg);
        }
    }

    pub fn dispatchmessage<'a>(&mut self, message: &mut SMSG, windowproc: &'a mut dyn TWindowProc) {
        unsafe {
            if !self.registereduserdata {
                self.registeruserdata();
            }

            self.setwindowproc(windowproc);
            winapi::um::winuser::DispatchMessageW(&mut message.msg);
            self.clearwindowproc();
        }
    }

    pub fn getclientrect(&self) -> Result<SRect, &'static str> {
        unsafe {
            let mut rect: RECT = mem::zeroed();
            let res = winapi::um::winuser::GetClientRect(self.window, &mut rect as LPRECT);
            if res == 0 {
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
        let mut point = winapi::shared::windef::POINT {
            x: point[0] as i32,
            y: point[1] as i32,
        };

        let success = unsafe { winapi::um::winuser::ScreenToClient(self.window, &mut point) };
        assert!(success != 0);

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
    // -- $$$FRK(TODO): right now this assumes a ton of defaults, we should pass those in
    // -- and move defaults to rustywindows
    pub fn createwindow(&self, title: &str, width: u32, height: u32) -> Result<SWindow, SErr> {
        unsafe {
            let windowstyle: DWORD = WS_OVERLAPPEDWINDOW;

            let screenwidth = GetSystemMetrics(SM_CXSCREEN);
            let screenheight = GetSystemMetrics(SM_CYSCREEN);

            let mut windowrect = RECT {
                left: 0,
                top: 0,
                right: width as LONG,
                bottom: height as LONG,
            };
            AdjustWindowRect(&mut windowrect, windowstyle, false as i32);

            let windowwidth = windowrect.right - windowrect.left;
            let windowheight = windowrect.bottom - windowrect.top;

            let windowx = cmp::max(0, (screenwidth - windowwidth) / 2);
            let windowy = cmp::max(0, (screenheight - windowheight) / 2);

            //self.class as ntdef::LPCWSTR,
            let windowclassnameparam = self.windowclassname.as_ptr() as ntdef::LPCWSTR;
            let mut titleparam: Vec<u16> = title.encode_utf16().collect();
            titleparam.push('\0' as u16);
            let hinstanceparam = self.winapi.hinstance;

            let hwnd: HWND = CreateWindowExW(
                0,
                windowclassnameparam,
                titleparam.as_ptr(),
                windowstyle,
                windowx,
                windowy,
                windowwidth,
                windowheight,
                ntdef::NULL as HWND,
                ntdef::NULL as HMENU,
                hinstanceparam,
                ntdef::NULL,
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
    C,
    D,
    Q,
    S,
    W,
    Space,
}

pub fn translatewmkey(key: winapi::shared::minwindef::WPARAM) -> EKey {
    match key {
        0x20 => EKey::Space,
        0x41 => EKey::A,
        0x43 => EKey::C,
        0x44 => EKey::D,
        0x51 => EKey::Q,
        0x53 => EKey::S,
        0x57 => EKey::W,
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
    Paint,
    Size,
    Input { raw_input: rawinput::SRawInput },
}

pub fn msgtype(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> EMsgType {
    match msg {
        winapi::um::winuser::WM_KEYDOWN => EMsgType::KeyDown {
            key: translatewmkey(wparam),
        },
        winapi::um::winuser::WM_KEYUP => EMsgType::KeyUp {
            key: translatewmkey(wparam),
        },
        winapi::um::winuser::WM_LBUTTONDOWN => EMsgType::LButtonDown {
            x_pos: winapi::shared::windowsx::GET_X_LPARAM(lparam),
            y_pos: winapi::shared::windowsx::GET_Y_LPARAM(lparam),
        },
        winapi::um::winuser::WM_LBUTTONUP => EMsgType::LButtonUp {
            x_pos: winapi::shared::windowsx::GET_X_LPARAM(lparam),
            y_pos: winapi::shared::windowsx::GET_Y_LPARAM(lparam),
        },
        winapi::um::winuser::WM_PAINT => EMsgType::Paint,
        winapi::um::winuser::WM_SIZE => EMsgType::Size,
        winapi::um::winuser::WM_INPUT => {
            const RI_SIZE : u32 = std::mem::size_of::<RAWINPUT>() as u32;
            const RI_HEADER_SIZE : u32 = std::mem::size_of::<RAWINPUTHEADER>() as u32;

            let mut bytes : [u8; RI_SIZE as usize] = [0; RI_SIZE as usize];

            unsafe {

                let result = GetRawInputData(
                    lparam as HRAWINPUT,
                    RID_INPUT,
                    &mut bytes[0] as *mut u8 as *mut winapi::ctypes::c_void,
                    &mut RI_SIZE,
                    RI_HEADER_SIZE
                );

                if result == std::u32::MAX {
                    panic!("Bad message.");
                }

                let raw : *mut RAWINPUT = &mut bytes[0] as *mut u8 as *mut RAWINPUT;

                let raw_input = rawinput::SRawInput::try_from(*raw).unwrap();

                EMsgType::Input { raw_input }

            }
        },
        _ => EMsgType::Invalid,
    }
}

pub fn debug_break() {
    unsafe { winapi::um::debugapi::DebugBreak() };
}

pub fn break_if_debugging() {
    if unsafe { winapi::um::debugapi::IsDebuggerPresent() == TRUE } {
        debug_break();
    }
}
