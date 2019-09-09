#![allow(dead_code)]

use safed3d12;
use safewindows;
use rustywindows;
use collections::{SPoolHandle, SPool};

use std::ptr;
use std::ops::{Deref, DerefMut};

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

pub struct SFactory {
    f: safed3d12::SFactory,
}

impl SFactory {
    pub fn create() -> Result<SFactory, &'static str> {
        Ok(SFactory {
            f: safed3d12::createdxgifactory4()?,
        })
    }

    pub fn raw(&self) -> &safed3d12::SFactory {
        &self.f
    }

    pub fn bestadapter(&self) -> Result<SAdapter, &'static str> {
        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let adapter1opt = self.f.enumadapters(adapteridx);
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

    pub fn createswapchain(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &mut safed3d12::SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {
        Ok(SSwapChain {
            sc: self
                .f
                .createswapchainforwindow(window, commandqueue, width, height)?,
            buffercount: 2,
            backbuffers: Vec::with_capacity(2),
        })
    }
}

pub struct SAdapter {
    a: safed3d12::SAdapter4,
}

impl SAdapter {
    pub fn createdevice(&mut self) -> Result<SDevice, &'static str> {
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

                match infoqueue.pushstoragefilter(&mut filter) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }
            None => {
                return Err("Could not get info queue from adapter.");
            }
        }

        Ok(SDevice { d: device })
    }
}

pub struct SActiveCommandAllocator {
    allocator: SPoolHandle,
    reusefencevalue: u64,
}

#[derive(Clone)]
pub struct SCommandList {
    allocator: SPoolHandle,
    list: safed3d12::SCommandList,
}

pub struct SCommandQueue<'device> {
    q: safed3d12::SCommandQueue<'device>,
    fence: SFence<'device>,
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,
    commandlisttype: safed3d12::ECommandListType,

    commandallocatorpool: SPool<safed3d12::SCommandAllocator<'device>>,
    activeallocators: Vec<SActiveCommandAllocator>,
    commandlistpool: SPool<SCommandList>,
}

impl<'device> Deref for SCommandQueue<'device> {
    type Target = safed3d12::SCommandQueue<'device>;

    fn deref(&self) -> &Self::Target {
        &self.q
    }
}

impl<'device> DerefMut for SCommandQueue<'device> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.q
    }
}

impl<'device> SCommandQueue<'device> {
    pub fn createcommandqueue(
        winapi: &safewindows::SWinAPI,
        device: &'device SDevice,
        commandlisttype: safed3d12::ECommandListType,
    ) -> Result<SCommandQueue<'device>, &'static str> {
        let qresult = device
            .raw()
            .createcommandqueue(safed3d12::ECommandListType::Direct)?;
        Ok(SCommandQueue {
            q: qresult,
            fence: device.createfence().unwrap(),
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
            commandlisttype: commandlisttype,

            commandallocatorpool: Default::default(),
            activeallocators: Vec::new(),
            commandlistpool: Default::default(),
        })
    }

    pub fn setup(&mut self, device: &'device SDevice, maxallocators: u16, maxcommandlists: u16) -> Result<(), &'static str> {
        let commandlisttype = self.commandlisttype;
        self.commandallocatorpool.setup(maxallocators, || {
            device.raw().createcommandallocator(commandlisttype).unwrap() // $$$FRK(TODO): need to find a way to not crash here
        });

        self.activeallocators.reserve(maxallocators as usize);

        let firstallocatorhandle = self.commandallocatorpool.handleforindex(0)?;
        let firstallocator = self.commandallocatorpool.getbyindex(0)?;

        self.commandlistpool.setup(maxcommandlists, || {
            SCommandList{
                allocator: firstallocatorhandle,
                list: device.raw().createcommandlist(firstallocator).unwrap(),
            }
        });

        for i in 0..maxcommandlists {
            let commandlist = self.commandlistpool.getbyindex(i)?;
            commandlist.list.close()?;
        }

        Ok(())
    }

    fn freeallocators(&mut self) {
        let completedvalue = self.fence.raw().getcompletedvalue();
        for alloc in &self.activeallocators {
            if alloc.reusefencevalue <= completedvalue {
                self.commandallocatorpool.pop(alloc.allocator);
            }
        }

        self.activeallocators.retain(|alloc| {
            alloc.reusefencevalue > completedvalue
        });
    }

    pub fn getunusedcommandlisthandle(&mut self) -> Result<SPoolHandle, &'static str> {
        self.freeallocators();

        if self.commandlistpool.full() || self.commandallocatorpool.full() {
            return Err("no available command list or allocator");
        }

        let commandallocatorhandle = self.commandallocatorpool.push()?;
        let commandallocator = self.commandallocatorpool.getmut(commandallocatorhandle)?;
        commandallocator.reset();

        let commandlisthandle = self.commandlistpool.push()?;
        let commandlist = self.commandlistpool.getmut(commandlisthandle)?;
        commandlist.list.reset(commandallocator)?;
        commandlist.allocator = commandallocatorhandle;

        Ok(commandlisthandle)
    }

    pub fn getcommandlist(&mut self, list: SPoolHandle) -> Result<&mut safed3d12::SCommandList, &'static str> {
        Ok(&mut (self.commandlistpool.getmut(list)?).list)
    }

    pub fn executecommandlist(&mut self, list: SPoolHandle) -> Result<(), &'static str> {
        #[allow(unused_assignments)]
        let mut allocator : SPoolHandle = Default::default();
        {
            let rawlist = self.commandlistpool.getmut(list)?;
            rawlist.list.close()?;
            self.q.executecommandlist(&mut rawlist.list);

            assert!(rawlist.allocator.valid());
            allocator = rawlist.allocator;
        }
        self.commandlistpool.pop(list);

        let fenceval = self.pushsignal()?;

        self.activeallocators.push(SActiveCommandAllocator{
            allocator: allocator,
            reusefencevalue: fenceval,
        });

        Ok(())
    }

    pub fn transitionresource(
        &mut self,
        list: SPoolHandle,
        resource: &safed3d12::SResource,
        beforestate: safed3d12::EResourceStates,
        afterstate: safed3d12::EResourceStates,
    ) -> Result<(), &'static str> {

        let commandlist = self.getcommandlist(list)?;
        let transbarrier = safed3d12::createtransitionbarrier(resource, beforestate, afterstate);
        commandlist.resourcebarrier(1, &[transbarrier]);
        Ok(())
    }

    pub fn clearrendertargetview(
        &mut self,
        list: SPoolHandle,
        rtvdescriptor: safed3d12::SDescriptorHandle,
        colour: &[f32; 4],
    ) ->Result<(), &'static str> {
        let commandlist = self.getcommandlist(list)?;
        commandlist.clearrendertargetview(rtvdescriptor, colour);
        Ok(())
    }

    pub fn pushsignal(&mut self) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        self.q.signal(&self.fence.raw(), self.nextfencevalue)
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

    pub fn rawqueue(&mut self) -> &mut safed3d12::SCommandQueue<'device> {
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
    pub fn rawmut(&mut self) -> &mut safed3d12::SDevice {
        &mut self.d
    }

    pub fn initrendertargetviews(
        &self,
        swap: &mut SSwapChain,
        heap: &SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap.backbuffers.is_empty());

        match heap.raw().type_ {
            safed3d12::EDescriptorHeapType::RenderTarget => {
                for backbuffidx in 0usize..2usize {
                    swap.backbuffers.push(swap.raw().getbuffer(backbuffidx)?);

                    let curdescriptorhandle = heap.cpuhandle(backbuffidx)?;
                    self.d.createrendertargetview(
                        &swap.backbuffers[backbuffidx as usize],
                        &curdescriptorhandle,
                    );
                }

                Ok(())
            }
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap."),
        }
    }

    pub fn createfence(&self) -> Result<SFence, &'static str> {
        Ok(SFence {
            f: self.d.createfence()?,
        })
    }

    pub fn createdescriptorheap(
        &self,
        type_: safed3d12::EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        //let raw = self.d.createdescriptorheap(type_, numdescriptors)?;
        Ok(SDescriptorHeap {
            dh: self.d.createdescriptorheap(type_, numdescriptors)?,
            numdescriptors: numdescriptors,
            descriptorsize: self.d.getdescriptorhandleincrementsize(type_),
            //cpudescriptorhandleforstart: raw.getcpudescriptorhandleforheapstart(),
        })
    }
}

pub struct SFence<'device> {
    f: safed3d12::SFence<'device>,
}

impl<'device> SFence<'device> {
    pub fn raw(&self) -> &safed3d12::SFence {
        &self.f
    }

    pub fn waitforvalue(
        &self,
        val: u64,
        event: &safewindows::SEventHandle,
        duration: u64,
    ) -> Result<(), &'static str> {
        if self.f.getcompletedvalue() < val {
            self.f.seteventoncompletion(val, event)?;
            event.waitforsingleobject(duration);
        }

        Ok(())
    }
}

pub struct SDescriptorHeap<'device> {
    dh: safed3d12::SDescriptorHeap<'device>,
    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: safed3d12::SDescriptorHandle<'heap, 'device>,
}

impl<'device> SDescriptorHeap<'device> {
    pub fn raw(&self) -> &safed3d12::SDescriptorHeap {
        &self.dh
    }

    pub fn cpuhandle(&self, index: usize) -> Result<safed3d12::SDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.dh.getcpudescriptorhandleforheapstart();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

pub struct SSwapChain {
    sc: safed3d12::SSwapChain,
    pub buffercount: u32,
    pub backbuffers: Vec<safed3d12::SResource>,
}

impl Deref for SSwapChain {
    type Target = safed3d12::SSwapChain;

    fn deref(&self) -> &Self::Target {
        &self.sc
    }
}

impl DerefMut for SSwapChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sc
    }
}

impl SSwapChain {
    pub fn raw(&self) -> &safed3d12::SSwapChain {
        &self.sc
    }
}

pub struct SD3D12Window<'device> {
    window: rustywindows::SWindow,
    pub swapchain: SSwapChain,
    curbuffer: usize,
    rtvdescriptorheap: SDescriptorHeap<'device>,
    curwidth: u32,
    curheight: u32,
}

pub fn createsd3d12window<'device>(
    factory: &mut SFactory,
    windowclass: &safewindows::SWindowClass,
    device: &'device SDevice,
    commandqueue: &mut safed3d12::SCommandQueue,
    title: &str,
    width: u32,
    height: u32,
) -> Result<SD3D12Window<'device>, &'static str> {

    let window = rustywindows::SWindow::create(windowclass, title, width, height).unwrap(); // $$$FRK(TODO): this panics, need to unify error handling
    let swapchain = factory.createswapchain(&window.raw(), commandqueue, width, height)?;
    let curbuffer = swapchain.raw().currentbackbufferindex();

    Ok(SD3D12Window{
        window: window,
        swapchain: swapchain,
        curbuffer: curbuffer,
        rtvdescriptorheap: device.createdescriptorheap(safed3d12::EDescriptorHeapType::RenderTarget, 10)?,
        curwidth: width,
        curheight: height,
    })
}

impl<'device> Deref for SD3D12Window<'device> {
    type Target = rustywindows::SWindow;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl<'device> DerefMut for SD3D12Window<'device> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl<'device> SD3D12Window<'device> {
    pub fn initrendertargetviews(&mut self, device: &'device SDevice) -> Result<(), &'static str> {
        device.initrendertargetviews(&mut self.swapchain, &self.rtvdescriptorheap)?;
        Ok(())
    }

    // -- $$$FRK(TODO): need to think about this, non-mut seems wrong (as does just handing out a pointer in general)
    pub fn currentbackbuffer(&self) -> &safed3d12::SResource {
        &self.swapchain.backbuffers[self.curbuffer]
    }

    pub fn currentbackbufferindex(&self) -> usize {
        self.curbuffer
    }

    pub fn currentrendertargetdescriptor(&self) -> Result<safed3d12::SDescriptorHandle, &'static str> {
        self.rtvdescriptorheap.cpuhandle(self.curbuffer)
    }

    pub fn present(&mut self) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): figure out what this value does
        let syncinterval = 1;
        self.swapchain.raw().present(syncinterval, 0)?;
        let newbuffer = self.swapchain.raw().currentbackbufferindex();
        assert!(newbuffer != self.curbuffer);
        self.curbuffer = newbuffer;

        Ok(())
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        commandqueue: &mut SCommandQueue,
        device: &'device SDevice,
    ) -> Result<(), &'static str> {
        if self.curwidth != width || self.curheight != height {
            let newwidth = std::cmp::max(1, width);
            let newheight = std::cmp::max(1, height);
            commandqueue.flushblocking()?;

            self.swapchain.backbuffers.clear();

            let desc = self.swapchain.raw().getdesc()?;
            self.swapchain
                .raw()
                .resizebuffers(2, newwidth, newheight, &desc)?;

            self.curbuffer = self.swapchain.currentbackbufferindex();
            self.initrendertargetviews(device)?;

            self.curwidth = newwidth;
            self.curheight = newheight;
        }

        Ok(())
    }

}