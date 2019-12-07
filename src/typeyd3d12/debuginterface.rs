use super::*;

pub struct SDebugInterface {
    debuginterface: ComPtr<ID3D12Debug>,
}

impl SDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            let mut result: SDebugInterface = mem::uninitialized();

            let riid = ID3D12Debug::uuidof();
            let voidcasted: *mut *mut c_void =
                &mut result.debuginterface as *mut _ as *mut *mut c_void;

            let hresult = D3D12GetDebugInterface(&riid, voidcasted);
            if winerror::SUCCEEDED(hresult) {
                Ok(result)
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
