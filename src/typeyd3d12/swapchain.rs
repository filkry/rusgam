use super::*;

#[derive(Clone)]
pub struct SSwapChain {
    swapchain: ComPtr<IDXGISwapChain4>,
}

impl SSwapChain {
    pub unsafe fn new_from_raw(raw: ComPtr<IDXGISwapChain4>) -> Self {
        Self {
            swapchain: raw,
        }
    }

    pub fn present(&self, syncinterval: u32, flags: u32) -> Result<(), &'static str> {
        let hr = unsafe { self.swapchain.Present(syncinterval, flags) };
        returnerrifwinerror!(hr, "Couldn't present to swap chain.");
        Ok(())
    }

    pub fn currentbackbufferindex(&self) -> usize {
        unsafe { self.swapchain.GetCurrentBackBufferIndex() as usize }
    }

    pub fn getbuffer(&self, idx: usize) -> Result<SResource, &'static str> {
        let mut rawbuf: *mut ID3D12Resource = ptr::null_mut();
        let hn = unsafe {
            self.swapchain.GetBuffer(
                idx as u32,
                &ID3D12Resource::uuidof(),
                &mut rawbuf as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(
            hn,
            "Couldn't get ID3D12Resource for backbuffer from swapchain."
        );

        Ok(unsafe { SResource::new_from_raw(ComPtr::from_raw(rawbuf)) })
    }

    pub fn getdesc(&self) -> Result<SSwapChainDesc, &'static str> {
        unsafe {
            let mut desc: DXGI_SWAP_CHAIN_DESC = mem::zeroed();
            let hr = self.swapchain.GetDesc(&mut desc as *mut _);
            returnerrifwinerror!(hr, "Couldn't get swap chain desc.");
            Ok(SSwapChainDesc { desc: desc })
        }
    }

    // -- $$$FRK(TODO): support correct params
    pub fn resizebuffers(
        &self,
        buffercount: u32,
        width: u32,
        height: u32,
        olddesc: &SSwapChainDesc,
    ) -> Result<(), &'static str> {
        unsafe {
            let hr = self.swapchain.ResizeBuffers(
                buffercount,
                width,
                height,
                olddesc.desc.BufferDesc.Format,
                olddesc.desc.Flags,
            );
            returnerrifwinerror!(hr, "Couldn't resize buffers.");
        }
        Ok(())
    }
}

pub struct SSwapChainDesc {
    desc: DXGI_SWAP_CHAIN_DESC,
}
