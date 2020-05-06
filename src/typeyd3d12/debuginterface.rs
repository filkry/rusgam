use super::*;

use winapi::shared::ntdef;
use winapi::um::dxgidebug::{IDXGIDebug};

pub struct SDebugInterface {
    debuginterface: ComPtr<ID3D12Debug>,
}

impl SDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            let mut raw_ptr = std::mem::MaybeUninit::<*mut ID3D12Debug>::uninit();

            let riid = ID3D12Debug::uuidof();
            let voidcasted: *mut *mut c_void = raw_ptr.as_mut_ptr() as *mut _ as *mut *mut c_void;

            let hresult = D3D12GetDebugInterface(&riid, voidcasted);
            if winerror::SUCCEEDED(hresult) {
                Ok(SDebugInterface {
                    debuginterface: ComPtr::from_raw(raw_ptr.assume_init()),
                })
            } else {
                Err("D3D12GetDebugInterface gave an error.")
            }
        }
    }

    pub fn enabledebuglayer(&self) -> () {
        unsafe {
            self.debuginterface.EnableDebugLayer();
        }
    }
}

pub struct SDXGIDebugInterface {
    debuginterface: ComPtr<IDXGIDebug>,
}

impl SDXGIDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            let mut strbytes : [i8; 100] = [0; 100];
            let mut curidx = 0;
            for ch in "dxgidebug".bytes() {
                strbytes[curidx] = ch as i8;
                curidx += 1;
            }

            let module = winapi::um::libloaderapi::GetModuleHandleA(strbytes.as_ptr());
            if module.is_null() {
                return Err("Failed to find dxgidebug.dll");
            }

            curidx = 0;
            for ch in "DXGIGetDebugInterface".bytes() {
                strbytes[curidx] = ch as i8;
                curidx += 1;
            }
            strbytes[curidx] = 0;

            let get_debug_interface_fn_ptr =
                winapi::um::libloaderapi::GetProcAddress(module, strbytes.as_ptr());
            if get_debug_interface_fn_ptr.is_null() {
                return Err("Failed to find DXGIGetDebugInterface");
            }

            let get_debug_interface_fn : extern "C" fn(winapi::shared::guiddef::REFIID, *mut *mut c_void) -> ntdef::HRESULT = std::mem::transmute(get_debug_interface_fn_ptr);

            let mut raw_ptr = std::mem::MaybeUninit::<*mut IDXGIDebug>::uninit();

            let riid = IDXGIDebug::uuidof();
            let voidcasted: *mut *mut c_void = raw_ptr.as_mut_ptr() as *mut _ as *mut *mut c_void;

            //let hresult = DXGIGetDebugInterface(&riid, voidcasted);
            let hresult = get_debug_interface_fn(&riid, voidcasted);
            if winerror::SUCCEEDED(hresult) {
                Ok(Self {
                    debuginterface: ComPtr::from_raw(raw_ptr.assume_init()),
                })
            } else {
                Err("DXGIGetDebugInterface gave an error.")
            }
        }
    }

    pub fn report_live_objects(&self) {
        // -- $$$FRK(TODO): support parameters?
        unsafe {
            self.debuginterface.ReportLiveObjects(
                winapi::um::dxgidebug::DXGI_DEBUG_ALL,
                winapi::um::dxgidebug::DXGI_DEBUG_RLO_ALL,
            );
        }
    }
}
