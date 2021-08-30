use super::*;

pub struct SAdapter1 {
    adapter: win::IDXGIAdapter1,
}

impl SAdapter1 {
    pub unsafe fn new_from_raw(raw: win::IDXGIAdapter1) -> Self {
        Self { adapter: raw }
    }

    pub fn getdesc(&self) -> win::DXGI_ADAPTER_DESC1 {
        unsafe {
            self.adapter.GetDesc1().unwrap()
        }
    }

    pub fn castadapter4(&self) -> Option<SAdapter4> {
        use win::Interface;
        match self.adapter.cast::<win::IDXGIAdapter4>() {
            Ok(a) => {
                return Some(SAdapter4 { adapter: a });
            }
            Err(_) => {
                return None;
            }
        };
    }

    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(win::IUnknown::from(self.adapter))
    }
}

pub struct SAdapter4 {
    adapter: win::IDXGIAdapter4,
}

impl SAdapter4 {
    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(win::IUnknown::from(self.adapter))
    }
}
