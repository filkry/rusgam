#![allow(dead_code)]

//use winapi::um::d3d12 as dx;
use std::{cmp, fmt, mem, ptr};
//use std::ptr::{null};

use winapi::{Interface};
use winapi::ctypes::{c_void};
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::dxgi1_4::*;
use winapi::shared::dxgi1_6::*;
use winapi::shared::{ntdef, winerror};
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::d3d12::*;
use winapi::um::{d3dcommon, d3d12sdklayers, errhandlingapi, libloaderapi, winnt, unknwnbase};
use winapi::um::winnt::LONG;
use winapi::um::winuser::*;

use wio::com::ComPtr;
//use winapi::shared::{guiddef};

trait ComPtrPtrs<T> {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T> where T: Interface {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown {
        self.as_raw() as *mut unknwnbase::IUnknown
    }
}

pub struct SErr {
    errcode: DWORD,
}

pub unsafe fn getlasterror() -> SErr {
    SErr{errcode: errhandlingapi::GetLastError()}
}


impl fmt::Debug for SErr {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        // -- $$$FRK(TODO): we can call GetLastError to impl Debug/Display for SErr
        Ok(())
    }
}

pub struct SDebugInterface {
    debuginterface: ComPtr<d3d12sdklayers::ID3D12Debug>,
}

pub fn getdebuginterface() -> Result<SDebugInterface, &'static str> {
    unsafe {
        let mut result: SDebugInterface = mem::uninitialized();

        let riid = d3d12sdklayers::ID3D12Debug::uuidof();
        let voidcasted: *mut *mut c_void = &mut result.debuginterface as *mut _ as *mut *mut c_void;

        let hresult = D3D12GetDebugInterface(&riid, voidcasted);
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
            self.debuginterface.EnableDebugLayer();
        }
    }
}

pub struct SWinAPI {
    hinstance: HINSTANCE,
}

pub fn initwinapi() -> Result<SWinAPI, SErr> {
    unsafe {
        let hinstance = libloaderapi::GetModuleHandleW(ntdef::NULL as *const u16);
        if !hinstance.is_null() {
            Ok(SWinAPI{hinstance: hinstance})
        }
        else {
            Err(getlasterror())
        }
    }
}

pub struct SWindowClass<'windows> {
    winapi: &'windows SWinAPI,
    windowclassname: &'static str,
    class: ATOM,
}

impl SWinAPI {
    pub fn registerclassex(&self,
                           windowclassname: &'static str) -> Result<SWindowClass, SErr> {
        unsafe {
            let classdata = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(DefWindowProcW), //wndproc,
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
                Ok(SWindowClass{winapi: self, windowclassname: windowclassname, class: atom})
            }
            else {
                Err(getlasterror())
            }
        }
    }
}

pub struct SWindow {
    window: HWND,
}

impl<'windows> SWindowClass<'windows> {
    pub fn createwindow(&self, title: &str, width: u32, height: u32) -> Result<SWindow, SErr> {
        unsafe {
            let windowstyle: DWORD = WS_OVERLAPPEDWINDOW;

            let screenwidth = GetSystemMetrics(SM_CXSCREEN);
            let screenheight = GetSystemMetrics(SM_CYSCREEN);

            let mut windowrect = RECT{left: 0, top: 0,
                                      right: width as LONG, bottom: height as LONG};
            AdjustWindowRect(&mut windowrect, windowstyle, false as i32);

            let windowwidth = windowrect.right - windowrect.left;
            let windowheight = windowrect.bottom - windowrect.top;

            let windowx = cmp::max(0, (screenwidth - windowwidth) / 2);
            let windowy = cmp::max(0, (screenheight - windowheight) / 2);

            //self.class as ntdef::LPCWSTR,
            let windowclassnameparam = self.windowclassname.as_ptr() as ntdef::LPCWSTR;
            let titleparam = title.as_ptr() as ntdef::LPCWSTR;
            let hinstanceparam = self.winapi.hinstance;

            let hwnd: HWND = CreateWindowExW(
                0,
                windowclassnameparam,
                titleparam,
                windowstyle,
                windowx,
                windowy,
                windowwidth,
                windowheight,
                ntdef::NULL as HWND,
                ntdef::NULL as HMENU,
                hinstanceparam,
                ntdef::NULL
            );

            if !hwnd.is_null() {
                Ok(SWindow{window: hwnd})
            }
            else {
                Err(getlasterror())
            }
         }
    }
}

pub struct SAdapter {
    adapter: ComPtr<IDXGIAdapter4>,
}

// -- $$$FRK(TODO): need to decide what I'm doing with errors re: HRESULT and DWORD errcodes -
// maybe a union?
pub fn getadapter() -> Result<SAdapter, &'static str> { 
    // $$$FRK(TODO): shouldn't pass debug flags in all builds
    let mut rawfactory: *mut IDXGIFactory4 = ptr::null_mut();
    let createfactoryresult = unsafe {
        CreateDXGIFactory2(DXGI_CREATE_FACTORY_DEBUG,
                           &IDXGIFactory4::uuidof(),
                           &mut rawfactory as *mut *mut _ as *mut *mut c_void)
    };
    if winerror::SUCCEEDED(createfactoryresult) {
        let factory: ComPtr<IDXGIFactory4> = unsafe { ComPtr::from_raw(rawfactory) };

        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();

            if unsafe { factory.EnumAdapters1(adapteridx, &mut rawadapter1) } ==
               winerror::DXGI_ERROR_NOT_FOUND {
                continue;
            }

            let mut adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };

            let mut adapterdesc: DXGI_ADAPTER_DESC1 = unsafe {mem::uninitialized() };
            unsafe { adapter1.GetDesc1(&mut adapterdesc) };

            if adapterdesc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
                continue;
            }

            let devicecreateresult = unsafe {
                D3D12CreateDevice(adapter1.asunknownptr(),
                                  d3dcommon::D3D_FEATURE_LEVEL_11_0,
                                  &ID3D12Device::uuidof(),
                                  ptr::null_mut()) };
            if !winerror::SUCCEEDED(devicecreateresult) {
                continue;
            }

            if adapterdesc.DedicatedVideoMemory > maxdedicatedmem {
                match adapter1.cast::<IDXGIAdapter4>() {
                    Ok(_) => {
                        bestadapter = adapteridx;
                        maxdedicatedmem = adapterdesc.DedicatedVideoMemory;
                    }
                    Err(_) => {}
                }
            }
        }

        if maxdedicatedmem > 0 {
            let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();
            unsafe { factory.EnumAdapters1(bestadapter, &mut rawadapter1) };
            let adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };
            match adapter1.cast::<IDXGIAdapter4>() {
                Ok(a) => {
                    return Ok(SAdapter{adapter: a});
                }
                Err(_) => {
                    return Err("Getting Adapter4 failed despite working earlier");
                }
            };
        }

        Err("Could not find valid adapter")
    }
    else {
        Err("CreateDXGIFactory2 gave an error.")
    }
}

