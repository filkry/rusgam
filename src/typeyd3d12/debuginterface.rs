use super::*;

pub struct SDebugInterface {
    debuginterface: ComPtr<ID3D12Debug>,
}

impl SDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            match D3D12GetDebugInterface::<ID3D12Debug>() {
                Ok(di) => Ok(SDebugInterface {
                    debuginterface: di,
                }),
                Err(_) => Err("D3D12GetDebugInterface gave an error."),
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
    debuginterface: IDXGIDebug,
}

impl SDXGIDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            let res = Win32::Graphics::Dxgi::DXGIGetDebugInterface1::<IDXGIDebug>(0);
            match res {
                Ok(di) => Ok(Self {
                    debuginterface: di,
                }),
                Err(_) => Err("DXGIGetDebugInterface gave an error."),
            }
            /*
            let mut strbytes : [i8; 100] = [0; 100];
            let mut curidx = 0;
            for ch in "dxgidebug".bytes() {
                strbytes[curidx] = ch as i8;
                curidx += 1;
            }

            let module = Win32::System::LibraryLoader::GetModuleHandleA(strbytes.as_ptr());
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
                Win32::System::LibraryLoader::GetProcAddress(module, strbytes.as_ptr());
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
            */
        }
    }

    pub fn report_live_objects(&self) {
        // -- $$$FRK(FUTURE WORK): support parameters?
        unsafe {
            self.debuginterface.ReportLiveObjects(
                Win32::Graphics::Dxgi::DXGI_DEBUG_ALL,
                Win32::Graphics::Dxgi::DXGI_DEBUG_RLO_ALL,
            );
        }
    }
}
