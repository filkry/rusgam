use super::*;

pub struct SAdapter1 {
    adapter: ComPtr<IDXGIAdapter1>,
}

impl SAdapter1 {
    pub unsafe fn new_from_raw(raw: ComPtr<IDXGIAdapter1>) -> Self {
        Self { adapter: raw }
    }

    pub fn getdesc(&self) -> DXGI_ADAPTER_DESC1 {
        let mut adapterdesc = mem::MaybeUninit::<DXGI_ADAPTER_DESC1>::uninit();
        unsafe {
            self.adapter.GetDesc1(adapterdesc.as_mut_ptr());
            return adapterdesc.assume_init();
        };
    }

    pub fn castadapter4(&self) -> Option<SAdapter4> {
        match self.adapter.cast::<IDXGIAdapter4>() {
            Ok(a) => {
                return Some(SAdapter4 { adapter: a });
            }
            Err(_) => {
                return None;
            }
        };
    }

    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(self.adapter.asunknownptr())
    }
}

pub struct SAdapter4 {
    adapter: ComPtr<IDXGIAdapter4>,
}

impl SAdapter4 {
    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(self.adapter.asunknownptr())
    }
}
