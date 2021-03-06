use super::*;

pub struct SFactory {
    factory: ComPtr<IDXGIFactory4>,
}

impl SFactory {
    pub fn new() -> Result<Self, &'static str> {
        let mut rawfactory: *mut IDXGIFactory4 = ptr::null_mut();
        let createfactoryresult = unsafe {
            CreateDXGIFactory2(
                DXGI_CREATE_FACTORY_DEBUG,
                &IDXGIFactory4::uuidof(),
                &mut rawfactory as *mut *mut _ as *mut *mut c_void,
            )
        };
        if winerror::SUCCEEDED(createfactoryresult) {
            return Ok(Self {
                factory: unsafe { ComPtr::from_raw(rawfactory) },
            });
        }

        Err("Couldn't get D3D12 factory.")
    }

    pub fn enumadapters(&self, adapteridx: u32) -> Option<SAdapter1> {
        let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();

        if unsafe { self.factory.EnumAdapters1(adapteridx, &mut rawadapter1) }
            == winerror::DXGI_ERROR_NOT_FOUND
        {
            return None;
        }

        let adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };
        Some(unsafe { SAdapter1::new_from_raw(adapter1) })
    }

    pub unsafe fn createswapchainforwindow(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {
        let buffercount = 2;

        let desc = SSwapChainDesc {
            width,
            height,
            format: EDXGIFormat::R8G8B8A8UNorm,
            stereo: false,
            sample_desc: SDXGISampleDesc {
                count: 1,
                quality: 0,
            },
            buffer_usage: SDXGIUsageFlags::RENDER_TARGET_OUTPUT,
            buffer_count: buffercount,
            scaling: EDXGIScaling::Stretch,
            swap_effect: EDXGISwapEffect::FlipSequential,
            alpha_mode: EDXGIAlphaMode::Unspecified,
            flags: SDXGISwapChainFlags::empty(),
        };

        let d3d_desc = desc.d3dtype();

        let mut rawswapchain: *mut IDXGISwapChain1 = ptr::null_mut();

        let hr = self.factory.CreateSwapChainForHwnd(
            commandqueue.raw().asunknownptr(),
            window.raw(),
            &d3d_desc,
            ptr::null(),
            ptr::null_mut(),
            &mut rawswapchain as *mut *mut _ as *mut *mut IDXGISwapChain1,
        );

        returnerrifwinerror!(hr, "Failed to create swap chain");

        let swapchain = ComPtr::from_raw(rawswapchain);

        match swapchain.cast::<IDXGISwapChain4>() {
            Ok(sc4) => Ok(SSwapChain::new_from_raw(sc4)),
            _ => Err("Swap chain could not be case to SwapChain4"),
        }
    }
}
