use super::*;

pub struct SDebugInterface {
    debuginterface: ComPtr<ID3D12Debug>,
}

impl SDebugInterface {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {

            let raw_ptr = std::mem::MaybeUninit::<*mut ID3D12Debug>::uninit();

            let riid = ID3D12Debug::uuidof();
            let voidcasted: *mut *mut c_void =
                raw_ptr.as_mut_ptr() as *mut _ as *mut *mut c_void;

            let hresult = D3D12GetDebugInterface(&riid, voidcasted);
            if winerror::SUCCEEDED(hresult) {
                Ok(SDebugInterface{
                    debuginterface: ComPtr::from_raw(raw_ptr.as_mut_ptr()),
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
