use super::*;

pub struct SFactory {
    factory: win::IDXGIFactory4,
}

impl SFactory {
    pub fn new() -> Result<Self, &'static str> {
        let createfactoryresult = unsafe {
            win::CreateDXGIFactory2::<win::IDXGIFactory4>(
                win::DXGI_CREATE_FACTORY_DEBUG,
            )
        };
        match createfactoryresult {
            Ok(rawfactory) => Ok(Self { factory: rawfactory, }),
            Err(_) => Err("Couldn't get D3D12 factory.")
        }
    }

    pub fn enumadapters(&self, adapteridx: u32) -> Option<SAdapter1> {
        let res = self.factory.EnumAdapters1(adapteridx);
        match res {
            Ok(rawadapter1) => Some(SAdapter1::new_from_raw(rawadapter1)),
            Err(_) => None,
        }
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

        let hr = self.factory.CreateSwapChainForHwnd(
            win::IUnknown::from(commandqueue.raw()),
            window.raw(),
            &d3d_desc,
            ptr::null(),
            None,
        );

        returnerrifwinerror!(hr, "Failed to create swap chain");

        let swapchain = hr.expect("checked err above");

        use win::Interface;

        match swapchain.cast::<win::IDXGISwapChain4>() {
            Ok(sc4) => Ok(SSwapChain::new_from_raw(sc4)),
            _ => Err("Swap chain could not be cast to SwapChain4"),
        }
    }
}
