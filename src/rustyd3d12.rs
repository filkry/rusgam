#![allow(dead_code)]
#![allow(unused_imports)]

use collections::{SPool, SPoolHandle};
use rustywindows;
use safed3d12;
use safewindows;
use directxgraphicssamples;

use std::ops::{Deref, DerefMut};
use std::ptr;
use std::cell::{RefCell, Ref, RefMut};

// -- $$$FRK(TODO): all these imports should not exist
use winapi::ctypes::c_void;
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;
use winapi::um::d3d12::D3D12_SUBRESOURCE_DATA;

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

    pub fn createswapchain<'w, 'cq>(
        &self,
        window: &'w safewindows::SWindow,
        commandqueue: &'cq safed3d12::SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain<'cq, 'w>, &'static str> {
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
pub struct SCommandList<'cl> {
    allocator: SPoolHandle,
    list: safed3d12::SCommandList<'cl>,
}

pub struct SCommandAllocatorPool<'device> {
    commandlisttype: safed3d12::ECommandListType,
    pool: RefCell<SPool<safed3d12::SCommandAllocator<'device>>>,
}

impl<'device> SCommandAllocatorPool<'device> {
    pub fn create(
        device: &'device SDevice,
        maxallocators: u16,
        commandlisttype: safed3d12::ECommandListType,
    ) -> Result<SCommandAllocatorPool<'device>, &'static str> {

        let pool = SPool::<safed3d12::SCommandAllocator>::create_with(
            maxallocators,
            || {
                device
                    .raw()
                    .createcommandallocator(commandlisttype)
                    .unwrap() // $$$FRK(TODO): need to find a way to not crash here
            },
        );

        Ok(SCommandAllocatorPool{
            commandlisttype: commandlisttype,
            pool: RefCell::new(pool),
        })
    }
}

impl<'device> Deref for SCommandAllocatorPool<'device> {
    type Target = RefCell<SPool<safed3d12::SCommandAllocator<'device>>>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

pub struct SCommandListPool<'allocator> {
    pool: RefCell<SPool<SCommandList<'allocator>>>,
}

impl<'allocator> Deref for SCommandListPool<'allocator> {
    type Target = RefCell<SPool<SCommandList<'allocator>>>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl<'allocator> SCommandListPool<'allocator> {
    pub fn create<'device>(
        device: &'device SDevice,
        maxcommandlists: u16,
        allocatorpool: &'allocator SCommandAllocatorPool<'device>,
    ) -> Result<SCommandListPool<'allocator>, &'static str> {

        let firstallocatorhandle = allocatorpool.borrow().handleforindex(0)?;
        let firstallocator = allocatorpool.borrow().getbyindex(0)?;

        let mut pool = SPool::<SCommandList>::create_with(
            maxcommandlists,
            || SCommandList {
                allocator: firstallocatorhandle,
                list: device.raw().createcommandlist(firstallocator).unwrap(),
            },
        );

        for i in 0..maxcommandlists {
            let commandlist = pool.getbyindex(i)?;
            commandlist.list.close()?;
        }

        Ok(SCommandListPool{
            pool: RefCell::new(pool),
        })
    }
}

pub struct SCommandQueue<'device, 'a, 'cl> {
    q: safed3d12::SCommandQueue<'device>,
    fence: SFence<'device>,
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,

    allocatorpool: &'a SCommandAllocatorPool<'device>,
    activeallocators: Vec<SActiveCommandAllocator>,
    commandlistpool: &'cl SCommandListPool<'a>
}

impl<'device, 'a, 'cl> Deref for SCommandQueue<'device, 'a, 'cl> {
    type Target = safed3d12::SCommandQueue<'device>;

    fn deref(&self) -> &Self::Target {
        &self.q
    }
}

impl<'device, 'a, 'cl> DerefMut for SCommandQueue<'device, 'a, 'cl> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.q
    }
}

pub struct SCommandQueueUpdateBufferResult<'r> {
    destination: safed3d12::SResource<'r>,
    intermediate: safed3d12::SResource<'r>,
}

impl<'device, 'a, 'cl> SCommandQueue<'device, 'a, 'cl> {
    pub fn createcommandqueue(
        winapi: &safewindows::SWinAPI,
        device: &'device SDevice,
        allocatorpool: &'a SCommandAllocatorPool<'device>,
        commandlistpool: &'cl SCommandListPool<'a>,
    ) -> Result<SCommandQueue<'device, 'a, 'cl>, &'static str> {
        let qresult = device
            .raw()
            .createcommandqueue(safed3d12::ECommandListType::Direct)?;
        Ok(SCommandQueue {
            q: qresult,
            fence: device.createfence().unwrap(),
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,

            allocatorpool: allocatorpool,
            activeallocators: Vec::new(),
            commandlistpool: commandlistpool,
        })
    }

    pub fn getunusedcommandlisthandle(&self) -> Result<SPoolHandle, &'static str> {
        self.freeallocators();

        let clpool = self.commandlistpool.borrow_mut();
        let alpool = self.allocatorpool.borrow_mut();

        if clpool.full() || alpool.full() {
            return Err("no available command list or allocator");
        }

        let commandallocatorhandle = alpool.push()?;
        let commandallocator = alpool.getmut(commandallocatorhandle)?;
        commandallocator.reset();

        let commandlisthandle = clpool.push()?;
        let commandlist = clpool.getmut(commandlisthandle)?;
        commandlist.list.reset(commandallocator)?;
        commandlist.allocator = commandallocatorhandle;

        Ok(commandlisthandle)
    }

    pub fn getcommandlist(
        &self,
        list: SPoolHandle,
    ) -> Result<&mut safed3d12::SCommandList<'a>, &'static str> {
        Ok(&mut (self.commandlistpool.borrow_mut().getmut(list)?).list)
    }

    pub fn executecommandlist(&self, list: SPoolHandle) -> Result<(), &'static str> {
        #[allow(unused_assignments)]
        let mut allocator: SPoolHandle = Default::default();
        {
            let rawlist = self.commandlistpool.borrow_mut().getmut(list)?;
            rawlist.list.close()?;
            self.q.executecommandlist(&mut rawlist.list);

            assert!(rawlist.allocator.valid());
            allocator = rawlist.allocator;
        }
        self.commandlistpool.borrow_mut().pop(list);

        let fenceval = self.pushsignal()?;

        self.activeallocators.push(SActiveCommandAllocator {
            allocator: allocator,
            reusefencevalue: fenceval,
        });

        Ok(())
    }

    pub fn freeallocators(&self) {
        let completedvalue = self.fence.raw().getcompletedvalue();
        for alloc in &self.activeallocators {
            if alloc.reusefencevalue <= completedvalue {
                self.allocatorpool.borrow_mut().pop(alloc.allocator);
            }
        }

        self.activeallocators
            .retain(|alloc| alloc.reusefencevalue > completedvalue);
    }

    pub fn transitionresource(
        &self,
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
        &self,
        list: SPoolHandle,
        rtvdescriptor: safed3d12::SDescriptorHandle,
        colour: &[f32; 4],
    ) -> Result<(), &'static str> {
        let commandlist = self.getcommandlist(list)?;
        commandlist.clearrendertargetview(rtvdescriptor, colour);
        Ok(())
    }

    pub fn pushsignal(&self) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        self.q.signal(&self.fence.raw(), self.nextfencevalue)
    }

    pub fn waitforfencevalue(&self, val: u64) {
        self.fence
            .waitforvalue(val, &self.fenceevent, <u64>::max_value())
            .unwrap();
    }

    pub fn flushblocking(&self) -> Result<(), &'static str> {
        let lastfencevalue = self.pushsignal()?;
        self.waitforfencevalue(lastfencevalue);
        Ok(())
    }

    pub fn rawqueue(&mut self) -> &mut safed3d12::SCommandQueue<'device> {
        &mut self.q
    }

    #[allow(unused_variables)]
    pub fn updatebufferresource<T>(
        &mut self,
        device: &'device mut SDevice,
        _list: SPoolHandle,
        bufferdata: &[T],
        flags: safed3d12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult<'device>, &'static str> {

        let buffersize = bufferdata.len() * std::mem::size_of::<T>();

        let mut destinationresource = device.raw().createcommittedresource(
            safed3d12::SHeapProperties::create(safed3d12::EHeapType::Default),
            safed3d12::EHeapFlags::ENone,
            safed3d12::SResourceDesc::createbuffer(buffersize, flags),
            safed3d12::EResourceStates::CopyDest,
            None,
        )?;

        // -- resource created with Upload type MUST have state GenericRead
        let mut intermediateresource = device.raw().createcommittedresource(
            safed3d12::SHeapProperties::create(safed3d12::EHeapType::Upload),
            safed3d12::EHeapFlags::ENone,
            safed3d12::SResourceDesc::createbuffer(buffersize, safed3d12::SResourceFlags::none()),
            safed3d12::EResourceStates::GenericRead,
            None,
        )?;

        // -- $$$FRK(TODO): move the rest of this to safed3d12?
        unsafe {
            let mut subresourcedata = D3D12_SUBRESOURCE_DATA{
                pData: bufferdata.as_ptr() as *const c_void,
                RowPitch: buffersize as isize,
                SlicePitch: buffersize as isize,
            };

            let commandlist = self.getcommandlist(_list)?;

            directxgraphicssamples::UpdateSubresourcesStack(
                commandlist.rawmut().as_raw(),
                destinationresource.raw_mut().as_raw(),
                intermediateresource.raw_mut().as_raw(),
                0,
                0,
                1,
                &mut subresourcedata);
        }

        Ok(SCommandQueueUpdateBufferResult{
            destination: destinationresource,
            intermediate: intermediateresource,
        })
    }
}

pub struct SBufferResourceResult<'r> {
    destinationresource: safed3d12::SResource<'r>,
    intermediateresource: safed3d12::SResource<'r>,
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

    pub fn initrendertargetviews<'cq, 'w>(
        &self,
        swap: &mut SSwapChain<'cq, 'w>,
        heap: &SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap.backbuffers.is_empty());

        match heap.raw().type_ {
            safed3d12::EDescriptorHeapType::RenderTarget => {
                for backbuffidx in 0usize..2usize {
                    let backbuffer : safed3d12::SResource<'cq> = swap.raw().getbuffer(backbuffidx)?;
                    swap.backbuffers.push(backbuffer);

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

pub struct SFence<'f> {
    f: safed3d12::SFence<'f>,
}

impl<'f> SFence<'f> {
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

pub struct SDescriptorHeap<'h> {
    dh: safed3d12::SDescriptorHeap<'h>,
    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: safed3d12::SDescriptorHandle<'heap, 'device>,
}

impl<'h> SDescriptorHeap<'h> {
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

pub struct SSwapChain<'cq, 'w> {
    sc: safed3d12::SSwapChain<'cq, 'w>,
    pub buffercount: u32,
    pub backbuffers: Vec<safed3d12::SResource<'cq>>,
}

impl<'cq, 'w> Deref for SSwapChain<'cq, 'w> {
    type Target = safed3d12::SSwapChain<'cq, 'w>;

    fn deref(&self) -> &Self::Target {
        &self.sc
    }
}

impl<'cq, 'w> DerefMut for SSwapChain<'cq, 'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sc
    }
}

impl<'cq, 'w> SSwapChain<'cq, 'w> {
    pub fn raw(&self) -> &safed3d12::SSwapChain<'cq, 'w> {
        &self.sc
    }
}

pub struct SD3D12Window<'w, 'cq, 'h> {
    window: &'w rustywindows::SWindow,
    pub swapchain: SSwapChain<'cq, 'w>,
    curbuffer: usize,
    rtvdescriptorheap: SDescriptorHeap<'h>,
    curwidth: u32,
    curheight: u32,
}

pub fn createsd3d12window<'w, 'device, 'cq>(
    factory: &mut SFactory,
    window: &'w mut rustywindows::SWindow,
    device: &'device SDevice,
    commandqueue: &'cq safed3d12::SCommandQueue,
) -> Result<SD3D12Window<'w, 'cq, 'device>, &'static str> {
    let width = window.width();
    let height = window.height();
    let swapchain = factory.createswapchain(&window.raw(), commandqueue, width, height)?;
    let curbuffer = swapchain.raw().currentbackbufferindex();

    Ok(SD3D12Window {
        window: window,
        swapchain: swapchain,
        curbuffer: curbuffer,
        rtvdescriptorheap: device
            .createdescriptorheap(safed3d12::EDescriptorHeapType::RenderTarget, 10)?,
        curwidth: width,
        curheight: height,
    })
}

impl<'w, 'cq, 'h> Deref for SD3D12Window<'w, 'cq, 'h> {
    type Target = rustywindows::SWindow;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl<'w, 'cq, 'h> SD3D12Window<'w, 'cq, 'h> {
    pub fn initrendertargetviews(&mut self, device: &SDevice) -> Result<(), &'static str> {
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

    pub fn currentrendertargetdescriptor(
        &self,
    ) -> Result<safed3d12::SDescriptorHandle, &'static str> {
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

    pub fn width(&self) -> u32 {
        self.curwidth
    }

    pub fn height(&self) -> u32 {
        self.curheight
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        commandqueue: &mut SCommandQueue,
        device: &SDevice,
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
