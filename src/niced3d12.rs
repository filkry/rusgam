#![allow(dead_code)]

use collections::{SStoragePool, SPoolHandle};
use directxgraphicssamples;
use rustywindows;
use safewindows;
use typeyd3d12;

use std::ops::{Deref, DerefMut};
use std::ptr;

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

// =================================================================================================
// MAIN TYPES
// =================================================================================================

// -- $$$FRK(TODO): it's possible these types need to be in the pools themselves, rather than just
// -- the raw typeyd3d12 types (then all the exposed types would just be handles + interfaces with
// -- safety constraints). As is, there is no way for internal references to metadata within these
// -- types

pub struct SD3D12Context {
    f: typeyd3d12::SFactory,

    // -- pools
    adapters: SStoragePool<typeyd3d12::SAdapter4>,
    swapchains: SStoragePool<typeyd3d12::SSwapChain>,
    devices: SStoragePool<typeyd3d12::SDevice>,
    descriptorheaps: SStoragePool<typeyd3d12::SDescriptorHeap>,
    commandallocators: SStoragePool<typeyd3d12::SCommandAllocator>,
    commandqueues: SStoragePool<typeyd3d12::SCommandQueue>,
    commandlists: SStoragePool<typeyd3d12::SCommandList>,
    resources: SStoragePool<typeyd3d12::SResource>,
    fences: SStoragePool<typeyd3d12::SFence>,
}

pub struct SAdapter {
    a: SPoolHandle,
}

pub struct SSwapChain {
    handle: SPoolHandle,
    pub buffercount: u32,

    // -- backbuffers SResources are allocated/freed by SSwapChain
    pub backbuffers: [SResource; 4], // -- max 4 backbuffers for now
}

pub struct SDevice {
    handle: SPoolHandle,
}

pub struct SCommandAllocator {
    handle: SPoolHandle,
}

pub struct SCommandList {
    handle: SPoolHandle,
    allocator: SPoolHandle,
}

pub struct SCommandQueue {
    handle: SPoolHandle,

    fence: SFence,
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,

    commandlisttype: typeyd3d12::ECommandListType,
}

pub enum EResourceMetadata {
    Invalid,
    SwapChainResource,
    BufferResource { count: usize, sizeofentry: usize },
}

#[derive(Default)]
pub struct SResource {
    handle: SPoolHandle,
    metadata: EResourceMetadata,
}

pub struct SFence {
    handle: SPoolHandle,
}

pub struct SDescriptorHeap {
    handle: SPoolHandle,
    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: typeyd3d12::SDescriptorHandle<'heap, 'device>,
}

pub struct SD3D12Window {
    window: rustywindows::SWindow,
    swapchain: SSwapChain, // owns associated swapchain

    curbuffer: usize,
    rtvdescriptorheap: SDescriptorHeap,
    curwidth: u32,
    curheight: u32,
}

// =================================================================================================
// HELPER TYPES
// =================================================================================================

pub struct SCommandQueueUpdateBufferResult {
    pub destinationresource: SPoolHandle,
    pub intermediateresource: SPoolHandle,
}

/*
pub struct SCommandPool {
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,
    commandlisttype: typeyd3d12::ECommandListType,

    // -- owned items
    fence: SFence,
    allocators: SPool<SCommandAllocator>,
    lists: SPool<SCommandList>,
}

impl SCommandPool {
    pub fn create(
        ctxt: &mut SD3D12Context,
        winapi: &safewindows::SWinAPI,
    ) -> Self {

        Self {
            fenceevent: winapi.createeventhandle().unwrap(),
        }
    }
}

pub struct SCommandPoolList {
    listhandle: SPoolHandle,
    allocatorhandle: SPoolHandle,
}
*/

// =================================================================================================
// IMPLS
// =================================================================================================

// -- $$FRK(TODO): almost every function in here should be unsafe
impl SD3D12Context {
    pub fn create() -> Result<Self, &'static str> {
        Ok(Self {
            f: typeyd3d12::createdxgifactory4()?,

            // -- $$$FRK(TODO): allow overriding these consts
            adapters: SStoragePool::<typeyd3d12::SAdapter4>::create(0, 1),
            swapchains: SStoragePool::<typeyd3d12::SSwapChain>::create(1, 1),
            devices: SStoragePool::<typeyd3d12::SDevice>::create(2, 4),
            descriptorheaps: SStoragePool::<typeyd3d12::SDescriptorHeap>::create(3, 10),
            commandallocators: SStoragePool::<typeyd3d12::SCommandAllocator>::create(4, 10),
            commandqueues: SStoragePool::<typeyd3d12::SCommandQueue>::create(5, 10),
            commandlists: SStoragePool::<typeyd3d12::SCommandList>::create(6, 10),
            resources: SStoragePool::<typeyd3d12::SResource>::create(7, 512),
            fences: SStoragePool::<typeyd3d12::SFence>::create(8, 10),
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
            let adapter1 = adapter1opt.expect("$$$FRK(TODO)");

            let adapterdesc = adapter1.getdesc();

            // -- $$$FRK(TODO): get rid of this d3d constant
            if adapterdesc.Flags & winapi::shared::dxgi::DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
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
            let adapter1 = self.f.enumadapters(bestadapter).expect("$$$FRK(TODO)");
            let adapter4 = adapter1.castadapter4().expect("$$$FRK(TODO)");

            let adapterhandle = self.adapters.insert_val(adapter4)?;
            return Ok(SAdapter{
                a: adapterhandle,
            });
        }

        Err("Could not find valid adapter")
    }

    pub unsafe fn create_swap_chain(
        &mut self,
        window: &safewindows::SWindow,
        commandqueue: SPoolHandle,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {

        let raw_command_queue = self.commandqueues.get(commandqueue)?;

        let newsc = self
            .f
            .createswapchainforwindow(window, raw_command_queue, width, height)?;
        let handle = self.swapchains.insert_val(newsc)?;

        Ok(SSwapChain {
            handle: handle,
            buffercount: 2,
            backbuffers: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        })
    }

    fn update_subresources_stack(
        &mut self,
        commandlist: &mut SCommandList,
        destinationresource: &mut SResource,
        intermediateresource: &mut SResource,
        intermediateoffset: u64,
        firstsubresource: u32,
        numsubresources: u32,
        srcdata: &mut typeyd3d12::SSubResourceData,
    ) {
        let rawcommandlist = self.commandlists.get_mut(commandlist.handle).unwrap();
        let mut rawdest = self.resources.get(destinationresource.handle).unwrap().clone();
        let mut rawintermediate = self.resources.get(intermediateresource.handle).unwrap().clone();

        unsafe {
            directxgraphicssamples::UpdateSubresourcesStack(
                rawcommandlist.rawmut().as_raw(),
                rawdest.raw_mut().as_raw(),
                rawintermediate.raw_mut().as_raw(),
                intermediateoffset,
                firstsubresource,
                numsubresources,
                srcdata.raw_mut(),
            );
        }
    }

    // ---------------------------------------------------------------------------------------------
    // Adapter functions
    // ---------------------------------------------------------------------------------------------

    pub unsafe fn create_device(&mut self, adapter: SPoolHandle) -> Result<SDevice, &'static str> {
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

        let handle = self.devices.insert_val(device)?;
        Ok(SDevice { handle: handle })
    }

    // ---------------------------------------------------------------------------------------------
    // Device functions
    // ---------------------------------------------------------------------------------------------

    pub unsafe fn create_fence(&mut self, device: SPoolHandle) -> Result<SFence, &'static str> {
        let rawdevice = self.devices.get(device)?;
        let fence = rawdevice.createfence()?;
        let handle = self.fences.insert_val(fence)?;
        Ok(SFence {
            handle: handle,
        })
    }

    pub unsafe fn create_render_target_view(
        &self,
        device: SPoolHandle,
        resource: SPoolHandle,
        dest_descriptor: &typeyd3d12::SDescriptorHandle,
    ) -> Result<(), &'static str> {
        let rawdevice = self.devices.get(device)?;
        let rawresource = self.resources.get(resource)?;

        rawdevice.createrendertargetview(rawresource, dest_descriptor);

        Ok(())
    }

    pub unsafe fn create_descriptor_heap(
        &mut self,
        device: SPoolHandle,
        type_: typeyd3d12::EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        //let raw = self.d.createdescriptorheap(type_, numdescriptors)?;

        let rawdevice = self.devices.get(device)?;

        let dh = rawdevice.create_descriptor_heap(type_, numdescriptors)?;
        let handle = self.descriptorheaps.insert_val(dh)?;

        Ok(SDescriptorHeap {
            handle: handle,
            numdescriptors: numdescriptors,
            descriptorsize: rawdevice.getdescriptorhandleincrementsize(type_),
            //cpudescriptorhandleforstart: raw.getcpudescriptorhandleforheapstart(),
        })
    }

    pub unsafe fn create_committed_buffer_resource<T>(
        &mut self,
        device: SPoolHandle,
        heaptype: typeyd3d12::EHeapType,
        flags: typeyd3d12::SResourceFlags,
        resourcestates: typeyd3d12::EResourceStates,
        bufferdata: &[T],
    ) -> Result<SResource, &'static str> {
        let rawdevice = self.devices.get(device)?;

        let buffersize = bufferdata.len() * std::mem::size_of::<T>();

        let destinationresource = rawdevice.createcommittedresource(
            typeyd3d12::SHeapProperties::create(heaptype),
            typeyd3d12::EHeapFlags::ENone,
            typeyd3d12::SResourceDesc::createbuffer(buffersize, flags),
            resourcestates,
            None,
        )?;

        let handle = self.resources.insert_val(destinationresource)?;

        Ok(SResource {
            handle: handle,
            metadata: EResourceMetadata::BufferResource {
                count: bufferdata.len(),
                sizeofentry: std::mem::size_of::<T>(),
            },
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Command List functions
    // ---------------------------------------------------------------------------------------------

    pub fn transition_resource(
        &self,
        list: SPoolHandle,
        resource: SPoolHandle,
        beforestate: typeyd3d12::EResourceStates,
        afterstate: typeyd3d12::EResourceStates,
    ) -> Result<(), &'static str> {
        let rawlist = self.commandlists.get(list)?;
        let rawresource = self.resources.get(resource)?;

        let transbarrier =
            typeyd3d12::createtransitionbarrier(rawresource, beforestate, afterstate);
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

    pub unsafe fn create_command_queue(
        &mut self,
        winapi: &safewindows::SWinAPI,
        device: SPoolHandle,
        commandlisttype: typeyd3d12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {

        let rawdevice = self.devices.get(device)?;

        let qresult = rawdevice.createcommandqueue(commandlisttype)?;
        let qhandle = self.commandqueues.insert_val(qresult)?;

        let fresult = rawdevice.createfence()?;
        let fhandle = self.fences.insert_val(fresult)?;

        Ok(SCommandQueue {
            handle: qhandle,
            fence: SFence{handle: fhandle},
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
            commandlisttype: commandlisttype,
        })
    }

    // -- $$$FRK(TODO): can the list know what queue it's on?
    pub unsafe fn execute_command_list(
        &self,
        queue: SPoolHandle,
        list: SPoolHandle,
    ) -> Result<(), &'static str> {
        //#[allow(unused_assignments)] $$$FRK(TODO): cleanup

        let rawqueue = self.commandqueues.get(queue)?;
        let rawlist = self.commandlists.get(list)?;

        rawlist.close()?;
        rawqueue.executecommandlist(rawlist);

        Ok(())
    }

    pub fn signal(
        &self,
        queue: SPoolHandle,
        fence: SPoolHandle,
        value: u64,
    ) -> Result<u64, &'static str> {
        let rawqueue = self.commandqueues.get(queue)?;
        let rawfence = self.fences.get(fence)?;
        rawqueue.signal(&rawfence, value)
    }

    // ---------------------------------------------------------------------------------------------
    // Fence functions
    // ---------------------------------------------------------------------------------------------

    pub fn fence_wait_for_value(
        &self,
        fence: SPoolHandle,
        fenceevent: &mut safewindows::SEventHandle,
        val: u64,
    ) {
        self.fence_wait_for_value_duration(fence, fenceevent, val, <u64>::max_value()).unwrap();
    }

    pub fn fence_wait_for_value_duration(
        &self,
        fence: SPoolHandle,
        fenceevent: &mut safewindows::SEventHandle,
        val: u64,
        duration: u64,
    ) -> Result<(), &'static str> {
        let rawfence = self.fences.get(fence)?;

        if rawfence.getcompletedvalue() < val {
            rawfence.seteventoncompletion(val, fenceevent)?;
            fenceevent.waitforsingleobject(duration);
        }

        Ok(())
    }

    // ---------------------------------------------------------------------------------------------
    // Descriptor Heap functions
    // ---------------------------------------------------------------------------------------------
    pub fn descriptor_heap_type(&self, heap: SPoolHandle) -> typeyd3d12::EDescriptorHeapType {
        let rawheap = self.descriptorheaps.get(heap).unwrap();
        rawheap.type_
    }

    pub fn descriptor_heap_cpu_handle_heap_start(
        &self,
        heap: SPoolHandle,
    ) -> typeyd3d12::SDescriptorHandle {
        let rawheap = self.descriptorheaps.get(heap).unwrap();
        rawheap.getcpudescriptorhandleforheapstart()
    }

    // ---------------------------------------------------------------------------------------------
    // Resource functions
    // ---------------------------------------------------------------------------------------------
    pub unsafe fn free_resource(&mut self, resource: SPoolHandle) {
        self.resources.free(resource);
    }

    pub fn create_vertex_buffer_view(
        &self,
        resource: SPoolHandle,
        count: usize,
        sizeofentry: usize,
    ) -> Result<typeyd3d12::SVertexBufferView, &'static str> {
        let rawresource = self.resources.get(resource)?;
        Ok(typeyd3d12::SVertexBufferView::create(
            rawresource.getgpuvirtualaddress(),
            (count * sizeofentry) as u32,
            sizeofentry as u32,
        ))
    }

    pub fn create_index_buffer_view(
        &self,
        resource: SPoolHandle,
        format: typeyd3d12::EFormat,
        count: usize,
        sizeofentry: usize,
    ) -> Result<typeyd3d12::SIndexBufferView, &'static str> {
        let rawresource = self.resources.get(resource)?;
        Ok(typeyd3d12::SIndexBufferView::create(
            rawresource.getgpuvirtualaddress(),
            format,
            (count * sizeofentry) as u32,
        ))
    }

    // ---------------------------------------------------------------------------------------------
    // Swap chain functions
    // ---------------------------------------------------------------------------------------------

    pub fn current_backbuffer_index(&self, swap_chain: SPoolHandle) -> Result<usize, &'static str> {
        let raw_sc = self.swapchains.get(swap_chain)?;
        Ok(raw_sc.currentbackbufferindex())
    }

    pub fn present(&self, swap_chain: SPoolHandle, sync_interval: u32, flags: u32) -> Result<(), &'static str> {
        let raw_sc = self.swapchains.get(swap_chain)?;
        raw_sc.present(sync_interval, flags)
    }

    pub fn swap_chain_get_desc(&self, swap_chain: SPoolHandle) -> Result<typeyd3d12::SSwapChainDesc, &'static str> {
        let raw_sc = self.swapchains.get(swap_chain)?;
        raw_sc.getdesc()
    }

    pub fn swap_chain_resize_buffers(
        &self,
        swap_chain: SPoolHandle,
        buffercount: u32,
        width: u32,
        height: u32,
        olddesc: &typeyd3d12::SSwapChainDesc,
    ) -> Result<(), &'static str> {
        let raw_sc = self.swapchains.get(swap_chain)?;
        raw_sc.resizebuffers(buffercount, width, height, olddesc)
    }
}

impl SAdapter {
    pub fn create_device(&mut self, ctxt: &mut SD3D12Context) -> Result<SDevice, &'static str> {
        unsafe { ctxt.create_device(self.a) }
    }
}

impl SSwapChain {
    pub fn current_backbuffer_index(&self, ctxt: &SD3D12Context) -> Result<usize, &'static str> {
        ctxt.current_backbuffer_index(self.handle)
    }

    pub fn present(&mut self, ctxt: &SD3D12Context, sync_interval: u32, flags: u32) -> Result<(), &'static str> {
        ctxt.present(self.handle, sync_interval, flags)
    }
}

impl SCommandList {
    pub fn update_buffer_resource<T>(
        &mut self,
        ctxt: &mut SD3D12Context,
        device: &mut SDevice,
        bufferdata: &[T],
        flags: typeyd3d12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {

        unsafe {

            let mut destinationresource = ctxt.create_committed_buffer_resource(
                device.handle,
                typeyd3d12::EHeapType::Default,
                flags,
                typeyd3d12::EResourceStates::CopyDest,
                bufferdata
            )?;

            // -- resource created with Upload type MUST have state GenericRead
            let mut intermediateresource = ctxt.create_committed_buffer_resource(
                device.handle,
                typeyd3d12::EHeapType::Upload,
                flags,
                typeyd3d12::EResourceStates::GenericRead,
                bufferdata
            )?;

            let mut srcdata = typeyd3d12::SSubResourceData::createbuffer(bufferdata);
            ctxt.update_subresources_stack(
                self,
                &mut destinationresource,
                &mut intermediateresource,
                0,
                0,
                1,
                &mut srcdata,
            );

            Ok(SCommandQueueUpdateBufferResult {
                destinationresource: destinationresource.handle,
                intermediateresource: intermediateresource.handle,
            })

        }
    }


}

impl Default for EResourceMetadata {
    fn default() -> Self {
        EResourceMetadata::Invalid
    }
}

impl SCommandQueue {
    pub fn create_command_queue(
        ctxt: &mut SD3D12Context,
        winapi: &safewindows::SWinAPI,
        device: &mut SDevice,
        commandlisttype: typeyd3d12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {
        unsafe { ctxt.create_command_queue(winapi, device.handle, commandlisttype) }
    }

    pub fn push_signal(&mut self, ctxt: &SD3D12Context) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        ctxt.signal(self.handle, self.fence.handle, self.nextfencevalue)
    }

    pub fn flush_blocking(&mut self, ctxt: &SD3D12Context) -> Result<(), &'static str> {
        let lastfencevalue = self.push_signal(ctxt)?;
        ctxt.fence_wait_for_value(self.fence.handle, &mut self.fenceevent, lastfencevalue);
        Ok(())
    }
}

impl typeyd3d12::SSubResourceData {
    pub fn createbuffer<T>(data: &[T]) -> Self {
        let buffersize = data.len() * std::mem::size_of::<T>();
        unsafe { Self::create(data.as_ptr(), buffersize, buffersize) }
    }
}

/*
pub struct SBufferResourceResult {
    destinationresource: typeyd3d12::SResource,
    intermediateresource: typeyd3d12::SResource,
}
*/

/*
impl<'heap> SDescriptorHandle<'heap> {
    pub fn offset(&mut self, count: u32) {
        let stride: usize = (count * self.heap.descriptorsize) as usize;
        self.handle.ptr += stride;
    }
}
*/

impl SDevice {
    pub fn init_render_target_views(
        &mut self,
        ctxt: &mut SD3D12Context,
        swap_chain: &mut SSwapChain,
        descriptor_heap: &mut SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap_chain.backbuffers.is_empty());

        match ctxt.descriptor_heap_type(descriptor_heap.handle) {
            typeyd3d12::EDescriptorHeapType::RenderTarget => {

                let raw_swap_chain = ctxt.swapchains.get(swap_chain.handle)?;

                for backbuffidx in 0usize..2usize {
                    let rawresource = raw_swap_chain.getbuffer(backbuffidx)?;
                    let handle = ctxt.resources.insert_val(rawresource)?;

                    let resource = SResource{
                        handle: handle,
                        metadata: EResourceMetadata::SwapChainResource,
                    };

                    swap_chain.backbuffers[backbuffidx] = resource;

                    let curdescriptorhandle = descriptor_heap.cpu_handle(ctxt, backbuffidx)?;
                    unsafe { ctxt.create_render_target_view(
                        self.handle,
                        handle,
                        &curdescriptorhandle,
                    )?; }
                }

                Ok(())
            }
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap."),
        }
    }

    pub fn create_descriptor_heap(
        &mut self,
        ctxt: &mut SD3D12Context,
        type_: typeyd3d12::EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        unsafe { ctxt.create_descriptor_heap(self.handle, type_, numdescriptors) }
    }
}

impl SDescriptorHeap {
    pub fn cpu_handle(
        &self,
        ctxt: &SD3D12Context,
        index: usize,
    ) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = ctxt.descriptor_heap_cpu_handle_heap_start(self.handle);
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

impl SResource {
    pub fn create_vertex_buffer_view(
        &mut self,
        ctxt: &SD3D12Context,
    ) -> Result<typeyd3d12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            ctxt.create_vertex_buffer_view(self.handle, count, sizeofentry)
        } else {
            Err("Trying to create vertexbufferview for non-buffer resource")
        }
    }

    pub fn create_index_buffer_view(
        &mut self,
        ctxt: &SD3D12Context,
        format: typeyd3d12::EFormat,
    ) -> Result<typeyd3d12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            ctxt.create_index_buffer_view(self.handle, format, count, sizeofentry)
        } else {
            Err("Trying to create indexbufferview for non-buffer resource")
        }
    }

    pub fn free(&mut self, ctxt: &mut SD3D12Context) {
        if self.handle.valid() {
            unsafe { ctxt.free_resource(self.handle); }
            self.metadata = EResourceMetadata::Invalid;
        }
    }
}

pub fn createsd3d12window(
    ctxt: &mut SD3D12Context,
    windowclass: &safewindows::SWindowClass,
    device: &mut SDevice,
    commandqueue: &mut SCommandQueue,
    title: &str,
    width: u32,
    height: u32,
) -> Result<SD3D12Window, &'static str> {
    let window = rustywindows::SWindow::create(windowclass, title, width, height).unwrap(); // $$$FRK(TODO): this panics, need to unify error handling

    unsafe {

        let swap_chain = ctxt.create_swap_chain(
            &window.raw(),
            commandqueue.handle,
            width,
            height
        )?;
        let cur_buffer = swap_chain.current_backbuffer_index(ctxt)?;

        let descriptor_heap = ctxt.create_descriptor_heap(
            device.handle,
            typeyd3d12::EDescriptorHeapType::RenderTarget,
            10
        )?;

        Ok(SD3D12Window {
            window: window,
            swapchain: swap_chain,
            curbuffer: cur_buffer,
            rtvdescriptorheap: descriptor_heap,
            curwidth: width,
            curheight: height,
        })

    }
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
    pub fn init_render_target_views(&mut self, ctxt: &mut SD3D12Context, device: &mut SDevice) -> Result<(), &'static str> {
        device.init_render_target_views(ctxt, &mut self.swapchain, &mut self.rtvdescriptorheap)?;
        Ok(())
    }

    // -- $$$FRK(TODO): need to think about this, non-mut seems wrong (as does just handing out a pointer in general)
    pub fn currentbackbuffer(&self) -> &SResource {
        &self.swapchain.backbuffers[self.curbuffer]
    }

    pub fn currentbackbufferindex(&self) -> usize {
        self.curbuffer
    }

    pub fn currentrendertargetdescriptor(
        &self,
        ctxt: &SD3D12Context,
    ) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
        self.rtvdescriptorheap.cpu_handle(ctxt, self.curbuffer)
    }

    pub fn present(&mut self, ctxt: &SD3D12Context) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): figure out what this value does
        let syncinterval = 1;
        self.swapchain.present(ctxt, syncinterval, 0)?;
        let newbuffer = self.swapchain.current_backbuffer_index(ctxt)?;
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
        ctxt: &mut SD3D12Context,
        width: u32,
        height: u32,
        commandqueue: &mut SCommandQueue,
        device: &mut SDevice,
    ) -> Result<(), &'static str> {
        if self.curwidth != width || self.curheight != height {
            let newwidth = std::cmp::max(1, width);
            let newheight = std::cmp::max(1, height);
            commandqueue.flush_blocking(ctxt)?;

            for backbuffer in self.swapchain.backbuffers.iter_mut() {
                backbuffer.free(ctxt);
            }

            let desc = ctxt.swap_chain_get_desc(self.swapchain.handle)?;
            ctxt.swap_chain_resize_buffers(
                self.swapchain.handle,
                2,
                newwidth,
                newheight,
                &desc,
            )?;

            self.curbuffer = self.swapchain.current_backbuffer_index(ctxt)?;
            self.init_render_target_views(ctxt, device)?;

            self.curwidth = newwidth;
            self.curheight = newheight;
        }

        Ok(())
    }
}
