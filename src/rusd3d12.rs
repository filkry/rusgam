//use winapi::um::d3d12 as dx;
use std::mem;
//use std::ptr::{null};
use winapi::{Interface};
use winapi::ctypes::{c_void};
use winapi::shared::{ntdef, windef, winerror};
use winapi::shared::minwindef::*;
use winapi::um::{d3d12, d3d12sdklayers, errhandlingapi, winnt};
use winapi::um::winuser::*;
//use winapi::shared::{guiddef};

#[allow(dead_code)]
pub struct SErr {
    errcode: DWORD,
}

pub unsafe fn getlasterror() -> SErr {
    SErr{errcode: errhandlingapi::GetLastError()}
}

// -- $$$FRK(TODO): we can call GetLastError to impl Debug/Display for SErr

#[allow(dead_code)]
pub struct SDebugInterface {
    debuginterface: *mut d3d12sdklayers::ID3D12Debug,
}

pub fn getdebuginterface() -> Result<SDebugInterface, &'static str> {
    unsafe {
        let mut result: SDebugInterface = mem::uninitialized();

        let riid = d3d12sdklayers::ID3D12Debug::uuidof();
        let voidcasted: *mut *mut c_void = &mut result.debuginterface as *mut _ as *mut *mut c_void;

        let hresult = d3d12::D3D12GetDebugInterface(&riid, voidcasted);
        if winerror::SUCCEEDED(hresult) {
            Ok(result)
        }
        else {
            Err("D3D12GetDebugInterface gave an error.")
        }
    }
}

impl SDebugInterface {
    pub fn enabledebuglayer(&self) -> () {
        unsafe {
            (*self.debuginterface).EnableDebugLayer();
        }
    }
}

#[allow(dead_code)]
pub struct SWindowClass {
    class: ATOM,
}

#[allow(dead_code)]
pub fn registerclassex(title: &str,
                       hinstance: HINSTANCE,
                       wndproc: WNDPROC) -> Result<SWindowClass, SErr> {
    unsafe {
        let classdata = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: wndproc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(hinstance, ntdef::NULL as *const u16),
            hCursor: LoadCursorW(ntdef::NULL as HINSTANCE, IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as windef::HBRUSH,
            lpszMenuName: ntdef::NULL as *const u16,
            lpszClassName: title.as_ptr() as *const winnt::WCHAR,
            hIconSm: ntdef::NULL as windef::HICON,
        };

        let atom = RegisterClassExW(&classdata);
        if atom > 0 {
            Ok(SWindowClass{class: atom})
        }
        else {
            Err(getlasterror())
        }
    }
}
