#![allow(dead_code)]
use std::{cmp, fmt, mem, ptr};
use std::convert::TryFrom;

use windows;
use winbindings::Windows::Win32;
use winbindings::Windows::Win32::Foundation;
use winbindings::Windows::Win32::System::Diagnostics::Debug::GetLastError;
use winbindings::Windows::Win32::System::LibraryLoader::GetModuleHandleW;
use winbindings::Windows::Win32::System::Performance;
use winbindings::Windows::Win32::UI::WindowsAndMessaging;

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
    unsafe fn asunknownptr(&mut self) -> *mut windows::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T>
where
    T: Interface,
{
    unsafe fn asunknownptr(&mut self) -> *mut windows::IUnknown {
        self.as_raw() as *mut windows::IUnknown
    }
}

pub struct SErr {
    errcode: DWORD,
}

pub unsafe fn getlasterror() -> SErr {
    SErr {
        errcode: GetLastError(),
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
        let hinstance = GetModuleHandleW(0);
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
            WindowsAndMessaging::UnregisterClassW(
                self.windowclassname,
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
    event: Foundation::HANDLE,
}

impl SEventHandle {
    pub unsafe fn raw(&self) -> Foundation::HANDLE {
        self.event
    }

    pub fn waitforsingleobject(&self, duration: u64) {
        unsafe { Win32::System::Threading::WaitForSingleObject(self.raw(), duration as DWORD) };
    }
}

impl SWinAPI {
    pub fn queryperformancecounter() -> i64 {
        let mut result : i64 = 0;
        let success = unsafe { Performance::QueryPerformanceCounter(&mut result) };
        if success == 0 {
            panic!("Can't query performance.");
        }

        result
    }

    pub unsafe fn queryperformancefrequencycounter() -> i64 {
        let mut result : i64 = 0;
        let success = Performance::QueryPerformanceFrequency(&mut result);
        if success == 0 {
            panic!("Can't query performance.");
        }

        result
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
                hIcon: LoadIconW(self.hinstance, 0),
                hCursor: LoadCursorW(0, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: 0 as *const u16,
                lpszClassName: windowclassname,
                hIconSm: 0 as HICON,
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
        let event = unsafe { Win32::System::Threading::CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null()) };

        if event == ptr::null() {
            return Err("Couldn't create event.");
        }

        Ok(SEventHandle { event: event })
    }

    pub fn get_cursor_pos(&self) -> [u32; 2] {
        let mut point = Foundation::POINT {
            x: 0,
            y: 0,
        };
        let success = unsafe { WindowsAndMessaging::GetCursorPos(&mut point) };
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
    msg: WindowsAndMessaging::MSG,
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
            let mut paintstruct = mem::MaybeUninit::<Win32::Graphics::Gdi::PAINTSTRUCT>::uninit();
            Win32::Graphics::Gdi::BeginPaint(self.window, paintstruct.as_mut_ptr());
        }
    }

    pub fn endpaint(&mut self) {
        unsafe {
            // -- $$$FRK(TODO): real paintstruct
            let mut paintstruct = mem::MaybeUninit::<Win32::Graphics::Gdi::PAINTSTRUCT>::uninit();
            Win32::Graphics::Gdi::EndPaint(self.window, paintstruct.as_mut_ptr());
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

            let mut raw_msg = mem::MaybeUninit::<WindowsAndMessaging::MSG>::zeroed();

            self.setwindowproc(windowproc);
            let foundmessage = WindowsAndMessaging::PeekMessageW(
                raw_msg.as_mut_ptr(),
                self.window,
                0,
                0,
                WindowsAndMessaging::PM_REMOVE,
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
            WindowsAndMessaging::TranslateMessage(&mut message.msg);
        }
    }

    pub fn dispatchmessage<'a>(&mut self, message: &mut SMSG, windowproc: &'a mut dyn TWindowProc) {
        unsafe {
            if !self.registereduserdata {
                self.registeruserdata();
            }

            self.setwindowproc(windowproc);
            WindowsAndMessaging::DispatchMessageW(&mut message.msg);
            self.clearwindowproc();
        }
    }

    pub fn getclientrect(&self) -> Result<SRect, &'static str> {
        unsafe {
            let mut rect: Foundation::RECT = mem::zeroed();
            let res = WindowsAndMessaging::GetClientRect(self.window, &mut rect as *mut Foundation::RECT);
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

pub fn translatewmkey(key: winapi::shared::minwindef::WPARAM) -> EKey {
    match key as i32 {
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
        winapi::um::winuser::VK_OEM_3 => EKey::Tilde,
        winapi::um::winuser::VK_TAB => EKey::Tab,
        winapi::um::winuser::VK_LEFT => EKey::LeftArrow,
        winapi::um::winuser::VK_RIGHT => EKey::RightArrow,
        winapi::um::winuser::VK_UP => EKey::UpArrow,
        winapi::um::winuser::VK_DOWN => EKey::DownArrow,
        winapi::um::winuser::VK_PRIOR => EKey::PageUp,
        winapi::um::winuser::VK_NEXT => EKey::PageDown,
        winapi::um::winuser::VK_HOME => EKey::Home,
        winapi::um::winuser::VK_END => EKey::End,
        winapi::um::winuser::VK_INSERT => EKey::Insert,
        winapi::um::winuser::VK_DELETE => EKey::Delete,
        winapi::um::winuser::VK_BACK => EKey::Backspace,
        winapi::um::winuser::VK_RETURN => EKey::Enter,
        winapi::um::winuser::VK_ESCAPE => EKey::Escape,
        winapi::um::winuser::VK_OEM_MINUS => EKey::Minus,
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
        winapi::um::winuser::WM_MBUTTONDOWN => EMsgType::MButtonDown {
            x_pos: winapi::shared::windowsx::GET_X_LPARAM(lparam),
            y_pos: winapi::shared::windowsx::GET_Y_LPARAM(lparam),
        },
        winapi::um::winuser::WM_MBUTTONUP => EMsgType::MButtonUp {
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

                #[allow(const_item_mutation)]
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
