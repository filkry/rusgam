#![allow(dead_code)]

use collections::{SPool, SPoolHandle};
use rustywindows;
use typeyd3d12;
use safewindows;
use directxgraphicssamples;

use std::ops::{Deref, DerefMut};
use std::ptr;

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

// =================================================================================================
// MAIN TYPES
// =================================================================================================

pub struct SD3D12Context {
    f: typeyd3d12::SFactory,

    // -- pools
    adapters: SPool<typeyd3d12::SAdapter4>,
    swapchains: SPool<typeyd3d12::SSwapChain>,
    devices: SPool<typeyd3d12::SDevice>,
}

pub struct SAdapter {
    a: SPoolHandle,
}

pub struct SSwapChain {
    sc: SPoolHandle,
    pub buffercount: u32,
    pub backbuffers: [SPoolHandle, 4], // -- max 4 backbuffers for now
}

pub struct SDevice {
    d: SPoolHandle,
}

pub struct SCommandList {
    list: SPoolHandle,
    allocator: SPoolHandle,
}

pub struct SCommandQueue {
    queue: SPoolHandle,

    fence: SPoolHandle,
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,

    commandlisttype: typeyd3d12::ECommandListType,
}

// =================================================================================================
// HELPER TYPES
// =================================================================================================

pub struct SCommandQueueUpdateBufferResult {
    pub destinationresource: SPoolHandle,
    pub intermediateresource: SPoolHandle,
}

// =================================================================================================
// IMPLS
// =================================================================================================

impl SD3D12Context {
    pub fn create() -> Result<Self, &'static str> {
        Ok(Self {
            f: typeyd3d12::createdxgifactory4()?,

            // -- $$$FRK(TODO): allow overriding these consts
            adapters: SPool<typeyd3d12::SAdapter4>::create(1),
            swapchains: SPool<typeyd3d12::SSwapChain>::create(1),
            devices: SPool<typeyd3d12::SDevice>::create(4),
            commandallocators: SPool<typeyd3d12::SCommandAllocator>::create(10),
            commandqueues: SPool<typeyd3d12::SCommandQueue>::create(10),
            commandlists: SPool<typeyd3d12::SCommandList>::create(10),
            resources: SPool<typeyd3d12::SResource>::create(512),
        })
    }

    pub fn create_best_adapter(&mut self) -> Result<SAdapter, &'static str> {
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
            return self.adapters.pushval(adapter4)?;
        }

        Err("Could not find valid adapter")
    }

    pub fn create_swap_chain(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &mut typeyd3d12::SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {

        let newsc = self.f.createswapchainforwindow(window, commandqueue, width, height)?;
        let handle = self.adapters.pushval(newsc)?;

        Ok(SSwapChain {
            sc: handle,
            buffercount: 2,
            backbuffers: [Default::default, 4],
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Adapter functions
    // ---------------------------------------------------------------------------------------------

    pub fn create_device(&mut self, adapter: SPoolHandle) -> Result<SPoolHandle, &'static str> {
        // -- $$$FRK(TODO): remove unwraps? Assert instead? Manual unwrap that asserts!
        let device = self.adapters.get(adapter)?.d3d12createdevice()?;

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

                // -- $$$FRK(DNS): need a struct version of this in typeyd3d12
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

        let handle = self.devices.pushval(device)?;
        Ok(SDevice { d: handle })
    }

    // ---------------------------------------------------------------------------------------------
    // Command List functions
    // ---------------------------------------------------------------------------------------------

    pub fn update_buffer_resource<T>(
        &mut self,
        commandlist: SPoolHandle,
        bufferdata: &[T],
        flags: typeyd3d12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {

        let mut destinationresource = device.createcommittedbufferresource(
            typeyd3d12::EHeapType::Default,
            flags,
            typeyd3d12::EResourceStates::CopyDest,
            bufferdata)?;

        // -- resource created with Upload type MUST have state GenericRead
        let mut intermediateresource = device.createcommittedbufferresource(
            typeyd3d12::EHeapType::Upload,
            flags,
            typeyd3d12::EResourceStates::GenericRead,
            bufferdata)?;

        let mut srcdata = typeyd3d12::SSubResourceData::createbuffer(bufferdata);
        updatesubresourcesstack(
            self,
            &mut destinationresource,
            &mut intermediateresource,
            0,
            0,
            1,
            &mut srcdata);

        Ok(SCommandQueueUpdateBufferResult{
            destination: destinationresource,
            intermediate: intermediateresource,
        })
    }

    pub fn transition_resource(
        &self,
        list: SPoolHandle,
        resource: SPoolHandle,
        beforestate: typeyd3d12::EResourceStates,
        afterstate: typeyd3d12::EResourceStates,
    ) -> Result<(), &'static str> {

        let rawlist = self.commandlists.get(list)?;
        let rawresource = self.resources.get(resource)?;

        let transbarrier = typeyd3d12::createtransitionbarrier(rawresource, beforestate, afterstate);
        rawlist.resourcebarrier(1, &[transbarrier]);
        Ok(())
    }

    pub fn clear_render_target_view(
        &self,
        list: SPoolHandle,
        rtvdescriptor: typeyd3d12::SDescriptorHandle,
        colour: &[f32; 4],
    ) -> Result<(), &'static str> {
        let rawlist = self.commandlists.get(list)?;
        rawlist.clearrendertargetview(rtvdescriptor, colour);
        Ok(())
    }

    // ---------------------------------------------------------------------------------------------
    // Command queue functions
    // ---------------------------------------------------------------------------------------------

    pub fn create_command_queue(
        &mut self,
        winapi: &safewindows::SWinAPI,
        device: SPoolHandle,
        commandlisttype: typeyd3d12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {
        let qresult = device
            .raw()
            .createcommandqueue(commandlisttype)?;
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

    // -- $$$FRK(TODO): can the list know what queue it's on?
    pub fn execute_command_list(&self, queue: SPoolHandle, list: SPoolHandle) -> Result<(), &'static str> {
        //#[allow(unused_assignments)] $$$FRK(TODO): cleanup

        let rawqueue = self.commandqueues.get(queue)?;
        let rawlist = self.commandlists.get(list)?;

        rawlist.close();
        rawqueue.executecommandlist(rawlist);

        Ok(())
    }

    pub fn signal(&self, queue: SPoolHandle, fence: SPoolHandle, value: u64) -> Result<u64, &'static str> {
        let rawqueue = self.commandqueues.get(queue)?;
        let rawfence = self.fences.get(fence)?;
        rawqueue.signal(&rawfence, value)
    }

    pub fn wait_for_fence_value(&self, queue: &SCommandQueue, val: u64) {
        let rawqueue = self.commandqueues.get(queue.queue)?;

        self.fence
            .waitforvalue(val, &self.fenceevent, <u64>::max_value())
            .unwrap();
    }

    pub fn flush_blocking(&self, queue: &mut SCommandQueue) -> Result<(), &'static str> {
        let lastfencevalue = self.push_signal(queue)?;
        self.wait_for_fence_value(queue, lastfencevalue);
        Ok(())
    }
}

impl SAdapter {
    pub fn createdevice(&self, ctxt: &mut SD3D12Context) -> Result<SPoolHandle, &'static str> {
        ctxt.createdevice(self)
    }
}

impl SCommandList {
    pub fn update_buffer_resource<T>(
        &self,
        ctxt: &SD3D12Context,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {
        ctxt.update_buffer_resource(self.list)
    }
}

impl SCommandQueue {
    pub fn push_signal(&mut self, ctxt: &SD3D12Context) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        ctxt.signal(self.queue, self.fence, self.nextfencevalue)
    }
}

impl typeyd3d12::SSubResourceData {
    pub fn createbuffer<T>(data: &[T]) -> Self {
        let buffersize = data.len() * std::mem::size_of::<T>();
        unsafe {
            Self::create(data.as_ptr(), buffersize, buffersize)
        }
    }
}

fn updatesubresourcesstack(
    commandlist: &mut SCommandList,
    destinationresource: &mut typeyd3d12::SResource,
    intermediateresource: &mut typeyd3d12::SResource,
    intermediateoffset: u64,
    firstsubresource: u32,
    numsubresources: u32,
    srcdata: &mut typeyd3d12::SSubResourceData,
) {
    unsafe {
        directxgraphicssamples::UpdateSubresourcesStack(
            commandlist.rawmut().as_raw(),
            destinationresource.raw_mut().as_raw(),
            intermediateresource.raw_mut().as_raw(),
            intermediateoffset,
            firstsubresource,
            numsubresources,
            srcdata.raw_mut());
    }
}

pub struct SBufferResourceResult {
    destinationresource: typeyd3d12::SResource,
    intermediateresource: typeyd3d12::SResource,
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

pub enum EResourceMetadata {
    Invalid,
    BufferResource {
        count: usize,
        sizeofentry: usize,
    },
}

pub struct SResource {
    r: typeyd3d12::SResource,
    metadata: EResourceMetadata,
}

impl Deref for SResource {
    type Target = typeyd3d12::SResource;

    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl DerefMut for SResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

impl SResource {
    pub fn createvertexbufferview(&self) -> Result<typeyd3d12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource{count, sizeofentry} = self.metadata {
            Ok(typeyd3d12::SVertexBufferView::create(
                self.r.getgpuvirtualaddress(),
                (count * sizeofentry) as u32,
                sizeofentry as u32,
            ))
        }
        else {
            Err("Trying to create vertexbufferview for non-buffer resource")
        }
    }

    pub fn createindexbufferview(&self, format: typeyd3d12::EFormat) -> Result<typeyd3d12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource{count, sizeofentry} = self.metadata {
            Ok(typeyd3d12::SIndexBufferView::create(
                self.r.getgpuvirtualaddress(),
                format,
                (count * sizeofentry) as u32,
            ))
        }
        else {
            Err("Trying to create indexbufferview for non-buffer resource")
        }
    }
}

impl SDevice {
    pub fn raw(&self) -> &typeyd3d12::SDevice {
        &self.d
    }
    pub fn rawmut(&mut self) -> &mut typeyd3d12::SDevice {
        &mut self.d
    }

    pub fn initrendertargetviews(
        &self,
        swap: &mut SSwapChain,
        heap: &SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap.backbuffers.is_empty());

        match heap.raw().type_ {
            typeyd3d12::EDescriptorHeapType::RenderTarget => {
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
        type_: typeyd3d12::EDescriptorHeapType,
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

    pub fn createcommittedbufferresource<T>(
        &self,
        heaptype: typeyd3d12::EHeapType,
        flags: typeyd3d12::SResourceFlags,
        resourcestates: typeyd3d12::EResourceStates,
        bufferdata: &[T]) -> Result<SResource, &'static str> {

        let buffersize = bufferdata.len() * std::mem::size_of::<T>();

        let destinationresource = self.d.createcommittedresource(
            typeyd3d12::SHeapProperties::create(heaptype),
            typeyd3d12::EHeapFlags::ENone,
            typeyd3d12::SResourceDesc::createbuffer(buffersize, flags),
            resourcestates,
            None,
        )?;
        Ok(SResource{
            r: destinationresource,
            metadata: EResourceMetadata::BufferResource {
                count: bufferdata.len(),
                sizeofentry: std::mem::size_of::<T>(),
            },
        })
    }
}

pub struct SFence {
    f: typeyd3d12::SFence,
}

impl SFence {
    pub fn raw(&self) -> &typeyd3d12::SFence {
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

pub struct SDescriptorHeap {
    dh: typeyd3d12::SDescriptorHeap,
    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: typeyd3d12::SDescriptorHandle<'heap, 'device>,
}

impl SDescriptorHeap {
    pub fn raw(&self) -> &typeyd3d12::SDescriptorHeap {
        &self.dh
    }

    pub fn cpuhandle(&self, index: usize) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.dh.getcpudescriptorhandleforheapstart();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

impl Deref for SSwapChain {
    type Target = typeyd3d12::SSwapChain;

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
    pub fn raw(&self) -> &typeyd3d12::SSwapChain {
        &self.sc
    }
}

pub struct SD3D12Window {
    window: rustywindows::SWindow,
    pub swapchain: SSwapChain,
    curbuffer: usize,
    rtvdescriptorheap: SDescriptorHeap,
    curwidth: u32,
    curheight: u32,
}

pub fn createsd3d12window(
    factory: &mut SFactory,
    windowclass: &safewindows::SWindowClass,
    device: &SDevice,
    commandqueue: &mut typeyd3d12::SCommandQueue,
    title: &str,
    width: u32,
    height: u32,
) -> Result<SD3D12Window, &'static str> {
    let window = rustywindows::SWindow::create(windowclass, title, width, height).unwrap(); // $$$FRK(TODO): this panics, need to unify error handling
    let swapchain = factory.createswapchain(&window.raw(), commandqueue, width, height)?;
    let curbuffer = swapchain.raw().currentbackbufferindex();

    Ok(SD3D12Window {
        window: window,
        swapchain: swapchain,
        curbuffer: curbuffer,
        rtvdescriptorheap: device
            .createdescriptorheap(typeyd3d12::EDescriptorHeapType::RenderTarget, 10)?,
        curwidth: width,
        curheight: height,
    })
}

impl Deref for SD3D12Window {
    type Target = rustywindows::SWindow;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl DerefMut for SD3D12Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl SD3D12Window {
    pub fn initrendertargetviews(&mut self, device: &SDevice) -> Result<(), &'static str> {
        device.initrendertargetviews(&mut self.swapchain, &self.rtvdescriptorheap)?;
        Ok(())
    }

    // -- $$$FRK(TODO): need to think about this, non-mut seems wrong (as does just handing out a pointer in general)
    pub fn currentbackbuffer(&self) -> &typeyd3d12::SResource {
        &self.swapchain.backbuffers[self.curbuffer]
    }

    pub fn currentbackbufferindex(&self) -> usize {
        self.curbuffer
    }

    pub fn currentrendertargetdescriptor(
        &self,
    ) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
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
