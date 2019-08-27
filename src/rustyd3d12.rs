use safed3d12;
use safewindows;

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::dxgi;
use winapi::shared::{dxgiformat, dxgitype, winerror};
use winapi::um::d3d12sdklayers::*;
use winapi::shared::minwindef::*;
use std::{ptr};

pub struct SFactory {
    f: safed3d12::SFactory,
}

impl SFactory {
    pub fn raw(&self) -> &safed3d12::SFactory {
        &self.f
    }

    pub fn bestadapter(&self) -> Result<SAdapter, &'static str> {
        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let mut adapter1opt = self.f.enumadapters(adapteridx);
            if let None = adapter1opt {
                continue;
            }
            let mut adapter1 = adapter1opt.expect("$$$FRK(TODO)");

            let adapterdesc = adapter1.getdesc();

            // -- $$$FRK(TODO): get rid of this d3d constant
            if adapterdesc.Flags & winapi::shared::dxgi::DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
                continue;
            }

            let devicecreateresult = adapter1.d3d12createdevice();
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
            let adapter1 = self.f.enumadapters(bestadapter).expect("$$$FRK(TODO)");
            let adapter4 = adapter1.castadapter4().expect("$$$FRK(TODO)");
            return Ok(SAdapter { a: adapter4 });
        }

        Err("Could not find valid adapter")
    }
}

pub struct SAdapter {
    a: safed3d12::SAdapter4,
}

impl SAdapter {
    pub fn createdevice(&mut self) -> Result<safed3d12::SDevice, &'static str> {
        // -- $$$FRK(TODO): remove unwraps
        let device = self.a.d3d12createdevice()?;

        // -- $$$FRK(TODO): debug only
        match device.castinfoqueue() {
            // -- $$$FRK(TODO): get rid of D3D enums?
            Some(infoqueue) => {
                infoqueue.setbreakonseverity(D3D12_MESSAGE_SEVERITY_CORRUPTION, TRUE);
                infoqueue.setbreakonseverity(D3D12_MESSAGE_SEVERITY_ERROR, TRUE);
                infoqueue.setbreakonseverity(D3D12_MESSAGE_SEVERITY_WARNING, TRUE);

                let mut suppressedseverities = [D3D12_MESSAGE_SEVERITY_INFO];

                let mut suppressedmessages =
                    [D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE];

                // -- $$$FRK(DNS): need a struct version of this in safed3d12
                let allowlist = D3D12_INFO_QUEUE_FILTER_DESC {
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: 0,
                    pSeverityList: ptr::null_mut(),
                    NumIDs: 0,
                    pIDList: ptr::null_mut(),
                };

                let denylist = D3D12_INFO_QUEUE_FILTER_DESC {
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: suppressedseverities.len() as u32,
                    pSeverityList: &mut suppressedseverities[0] as *mut u32,
                    NumIDs: suppressedmessages.len() as u32,
                    pIDList: &mut suppressedmessages[0] as *mut u32,
                };

                let mut filter = D3D12_INFO_QUEUE_FILTER {
                    AllowList: allowlist,
                    DenyList: denylist,
                };

                match infoqueue.pushstoragefilter(&mut filter)  {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }
            None => {
                return Err("Could not get info queue from adapter.");
            }
        }

        Ok(safed3d12::SDevice { device: device })
    }
}

pub struct SCommandQueue<'device> {
    q: safed3d12::SCommandQueue<'device>,
    fence: safed3d12::SFence<'device>,
    fenceevent: safewindows::SEventHandle,
    nextfencevalue: u64,
}

impl<'device> SCommandQueue<'device> {
    pub fn createcommandqueue(
        winapi: &safewindows::SWinAPI,
        device: &mut safed3d12::SDevice,
    ) -> Result<SCommandQueue<'device>, &'static str> {
        let qresult = device
            .createcommandqueue(safed3d12::ECommandListType::Direct)
            .unwrap();
        Ok(SCommandQueue {
            q: qresult,
            fence: device.createfence().unwrap(),
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
        })
    }

    pub fn pushsignal(&mut self) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        self.q.pushsignal(&self.fence, self.nextfencevalue)
    }

    pub fn waitforfencevalue(&self, val: u64) {
        self.fence
            .waitforvalue(val, &self.fenceevent, <u64>::max_value())
            .unwrap();
    }

    pub fn flushblocking(&mut self) -> Result<(), &'static str> {
        let lastfencevalue = self.pushsignal()?;
        self.waitforfencevalue(lastfencevalue);
        Ok(())
    }

    pub fn rawqueue(&mut self) -> &mut safed3d12::SCommandQueue {
        &mut self.q
    }
}


// --
/*
impl<'heap> SDescriptorHandle<'heap> {
    pub fn offset(&mut self, count: u32) {
        let stride: usize = (count * self.heap.descriptorsize) as usize;
        self.handle.ptr += stride;
    }
}
*/


pub struct SDevice {
    d: safed3d12::SDevice,
}

impl SDevice {
    pub fn raw(&self) -> &safed3d12::SDevice {
        &self.d
    }
    pub fn rawmut(&self) -> &mut safed3d12::SDevice {
        &mut self.d
    }

    pub fn initrendertargetviews(
        &self,
        swap: &SSwapChain,
        heap: &SDescriptorHeap,
    ) -> Result<(), &'static str> {
        match heap.type_ {
            EDescriptorHeapType::RenderTarget => {
                let mut curdescriptorhandle = heap.cpuhandle(0);

                for backbuf in &swap.backbuffers {
                    unsafe {
                        self.device.CreateRenderTargetView(
                            backbuf.resource.as_raw(),
                            ptr::null(),
                            curdescriptorhandle.handle,
                        );
                    };

                    curdescriptorhandle.offset(1);
                    //curdescriptorhandle.ptr += heap.descriptorsize as usize;
                    //curdescriptorhandle.Offset(descriptorsize);
                }

                Ok(())
            }
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap."),
        }
    }
}

/*
impl SFence {
    #[allow(unused_variables)]
    pub fn waitforvalue(
        &self,
        val: u64,
        event: &safewindows::SEventHandle,
        duration: u64,
    ) -> Result<(), &'static str> {
        if unsafe { self.fence.GetCompletedValue() } < val {
            let hn = unsafe { self.fence.SetEventOnCompletion(val, event.raw()) };
            returnerrifwinerror!(hn, "Could not set fence event on completion");
            unsafe { synchapi::WaitForSingleObject(event.raw(), duration as DWORD) };
        }

        Ok(())
    }
}
*/