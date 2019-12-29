#![allow(dead_code)]

use std::{cmp, fmt, mem, ptr};

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
    Paint,
    Size,
    Input,
}

pub fn msgtype(msg: UINT, wparam: WPARAM, _lparam: LPARAM) -> EMsgType {
    match msg {
        winapi::um::winuser::WM_KEYDOWN => EMsgType::KeyDown {
            key: translatewmkey(wparam),
        },
        winapi::um::winuser::WM_KEYUP => EMsgType::KeyUp {
            key: translatewmkey(wparam),
        },
        winapi::um::winuser::WM_PAINT => EMsgType::Paint,
        winapi::um::winuser::WM_SIZE => EMsgType::Size,
        winapi::um::winuser::WM_INPUT => EMsgType::Input,
        _ => EMsgType::Invalid,
    }
}
