use super::*;

use winbindings::Windows::Win32::Graphics::Dxgi;

pub struct SFactory {
    raw: t12::SFactory,
}

impl SFactory {
    pub fn create() -> Result<Self, &'static str> {
        Ok(Self {
            raw: t12::SFactory::new()?,
        })
    }

    pub fn create_best_adapter(&mut self) -> Result<SAdapter, &'static str> {
        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let adapter1opt = self.raw.enumadapters(adapteridx);
            if let None = adapter1opt {
                continue;
            }
            let adapter1 = adapter1opt.expect("Couldn't get a graphics adapter");

            let adapterdesc = adapter1.getdesc();

            if adapterdesc.Flags & Dxgi::DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
                continue;
            }

            let devicecreateresult = unsafe { adapter1.d3d12createdevice() };
            if let Err(_) = devicecreateresult {
                continue;
            }

            if adapterdesc.DedicatedVideoMemory > maxdedicatedmem {
                match adapter1.castadapter4() {
                    Some(_) => {
                        bestadapter = adapteridx;
                        maxdedicatedmem = adapterdesc.DedicatedVideoMemory;
                    }
                    None => {}
                }
            }
        }

        if maxdedicatedmem > 0 {
            let adapter1 = self.raw.enumadapters(bestadapter).expect("couldn't get graphics adapter");
            let adapter4 = adapter1.castadapter4().expect("system does not support D3D API level providing Adapter4");

            return Ok(SAdapter::new_from_raw(adapter4));
        }

        Err("Could not find valid adapter")
    }

    pub fn create_swap_chain(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {
        let newsc = unsafe {
            self.raw
                .createswapchainforwindow(window, commandqueue.raw(), width, height)?
        };

        Ok(SSwapChain::new_from_raw(newsc, 2))
    }
}
