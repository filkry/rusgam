#![allow(dead_code)]

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

pub struct SFactory {
    raw: typeyd3d12::SFactory,
}

pub struct SAdapter {
    raw: typeyd3d12::SAdapter4,
}

pub struct SDevice {
    raw: typeyd3d12::SDevice,
}

pub struct SSwapChain {
    raw: typeyd3d12::SSwapChain,

    pub buffercount: u32,
    pub backbuffers: Vec<SResource>, // $$FRK(TODO): VecArray PLEASE
}

pub struct SCommandList {
    raw: typeyd3d12::SCommandList,
    //allocator: &RefCell<typeyd3d12::SCommandAllocator>,
}

pub struct SCommandQueue {
    raw: typeyd3d12::SCommandQueue,

    fence: SFence,
    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,

    commandlisttype: typeyd3d12::ECommandListType,
}

pub struct SFence {
    raw: typeyd3d12::SFence,
}

pub enum EResourceMetadata {
    Invalid,
    SwapChainResource,
    BufferResource { count: usize, sizeofentry: usize },
}

pub struct SResource {
    raw: typeyd3d12::SResource,

    metadata: EResourceMetadata,
}

pub struct SDescriptorHeap {
    raw: typeyd3d12::SDescriptorHeap,

    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: typeyd3d12::SDescriptorHandle<'heap, 'device>,
}

pub struct SD3D12Window {
    window: rustywindows::SWindow,
    swapchain: SSwapChain,

    curbuffer: usize,
    rtvdescriptorheap: SDescriptorHeap,
    curwidth: u32,
    curheight: u32,
}

// =================================================================================================
// HELPER TYPES
// =================================================================================================

pub struct SCommandQueueUpdateBufferResult {
    pub destinationresource: SResource,
    pub intermediateresource: SResource,
}

// =================================================================================================
// IMPLS
// =================================================================================================

// -- $$FRK(TODO): almost every function in here should be unsafe
impl SFactory {
    pub fn create() -> Result<Self, &'static str> {
        Ok(Self {
            raw: typeyd3d12::createdxgifactory4()?,
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
            let adapter1 = self.raw.enumadapters(bestadapter).expect("$$$FRK(TODO)");
            let adapter4 = adapter1.castadapter4().expect("$$$FRK(TODO)");

            return Ok(SAdapter{ raw: adapter4, });
        }

        Err("Could not find valid adapter")
    }

    pub fn create_swap_chain(
        &mut self,
        window: &safewindows::SWindow,
        commandqueue: &mut SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {

        let newsc = unsafe { self
            .raw
            .createswapchainforwindow(window, &commandqueue.raw, width, height)? };

        Ok(SSwapChain {
            raw: newsc,
            buffercount: 2,
            backbuffers: Vec::with_capacity(2),
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Adapter functions
// ---------------------------------------------------------------------------------------------
impl SAdapter {
    pub fn create_device(&mut self) -> Result<SDevice, &'static str> {
        // -- $$$FRK(TODO): remove unwraps? Assert instead? Manual unwrap that asserts!
        let device = unsafe { self.raw.d3d12createdevice()? };

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

        Ok(SDevice { raw: device })
    }
}

// ---------------------------------------------------------------------------------------------
// Device functions
// ---------------------------------------------------------------------------------------------
impl SDevice {

    pub fn create_fence(&mut self) -> Result<SFence, &'static str> {
        let fence = self.raw.createfence()?;
        Ok(SFence {
            raw: fence,
        })
    }

    pub fn create_render_target_view(
        &mut self,
        render_target_resource: &mut SResource,
        dest_descriptor: &typeyd3d12::SDescriptorHandle,
    ) -> Result<(), &'static str> {
        self.raw.createrendertargetview(&render_target_resource.raw, dest_descriptor);
        Ok(())
    }

    pub fn create_descriptor_heap(
        &mut self,
        type_: typeyd3d12::EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        //let raw = self.d.createdescriptorheap(type_, numdescriptors)?;

        let dh = self.raw.create_descriptor_heap(type_, numdescriptors)?;

        Ok(SDescriptorHeap {
            raw: dh,
            numdescriptors: numdescriptors,
            descriptorsize: self.raw.getdescriptorhandleincrementsize(type_),
            //cpudescriptorhandleforstart: raw.getcpudescriptorhandleforheapstart(),
        })
    }

    pub unsafe fn create_committed_buffer_resource<T>(
        &self, // verified thread safe via docs
        heaptype: typeyd3d12::EHeapType,
        flags: typeyd3d12::SResourceFlags,
        resourcestates: typeyd3d12::EResourceStates,
        bufferdata: &[T],
    ) -> Result<SResource, &'static str> {
        let buffersize = bufferdata.len() * std::mem::size_of::<T>();

        let destinationresource = self.raw.createcommittedresource(
            typeyd3d12::SHeapProperties::create(heaptype),
            typeyd3d12::EHeapFlags::ENone,
            typeyd3d12::SResourceDesc::createbuffer(buffersize, flags),
            resourcestates,
            None,
        )?;

        Ok(SResource {
            raw: destinationresource,
            metadata: EResourceMetadata::BufferResource {
                count: bufferdata.len(),
                sizeofentry: std::mem::size_of::<T>(),
            },
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Command List functions
// ---------------------------------------------------------------------------------------------
impl SCommandList {

    pub fn transition_resource(
        &mut self,
        resource: &SResource,
        beforestate: typeyd3d12::EResourceStates,
        afterstate: typeyd3d12::EResourceStates,
    ) -> Result<(), &'static str> {
        let transbarrier =
            typeyd3d12::createtransitionbarrier(&resource.raw, beforestate, afterstate);
        self.raw.resourcebarrier(1, &[transbarrier]);
        Ok(())
    }

    pub fn clear_render_target_view(
        &mut self,
        rtvdescriptor: typeyd3d12::SDescriptorHandle,
        colour: &[f32; 4],
    ) -> Result<(), &'static str> {
        self.raw.clearrendertargetview(rtvdescriptor, colour);
        Ok(())
    }

}

// ---------------------------------------------------------------------------------------------
// Command queue functions
// ---------------------------------------------------------------------------------------------

impl SCommandQueue {

    pub fn create(
        device: &mut SDevice,
        winapi: &safewindows::SWinAPI,
        commandlisttype: typeyd3d12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {

        let qresult = device.raw.createcommandqueue(commandlisttype)?;

        Ok(SCommandQueue {
            raw: qresult,
            fence: device.create_fence()?,
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
            commandlisttype: commandlisttype,
        })
    }

    pub fn execute_command_list(
        &self, // -- verified thread safe in docs
        list: &mut SCommandList,
    ) -> Result<(), &'static str> {
        list.raw.close()?;
        unsafe { self.raw.executecommandlist(&list.raw) };
        Ok(())
    }

    pub fn signal(
        &self, // -- I'm assuming this is safe
        fence: &SFence,
        value: u64,
    ) -> Result<u64, &'static str> {
        self.raw.signal(&fence.raw, value)
    }

}

// ---------------------------------------------------------------------------------------------
// Fence functions
// ---------------------------------------------------------------------------------------------
impl SFence {

    pub fn wait_for_value(
        &self,
        fenceevent: &mut safewindows::SEventHandle,
        val: u64,
    ) {
        self.wait_for_value_duration(fenceevent, val, <u64>::max_value()).unwrap();
    }

    pub fn wait_for_value_duration(
        &self,
        fenceevent: &mut safewindows::SEventHandle,
        val: u64,
        duration: u64,
    ) -> Result<(), &'static str> {
        if self.raw.getcompletedvalue() < val {
            self.raw.seteventoncompletion(val, fenceevent)?;
            fenceevent.waitforsingleobject(duration);
        }

        Ok(())
    }

}

// ---------------------------------------------------------------------------------------------
// Descriptor Heap functions
// ---------------------------------------------------------------------------------------------

impl SDescriptorHeap {
    pub fn type_(&self) -> typeyd3d12::EDescriptorHeapType {
        self.raw.type_
    }

    pub fn cpu_handle_heap_start(
        &self,
    ) -> typeyd3d12::SDescriptorHandle {
        self.raw.getcpudescriptorhandleforheapstart()
    }
}

// ---------------------------------------------------------------------------------------------
// Resource functions
// ---------------------------------------------------------------------------------------------

impl SResource {

    pub fn create_vertex_buffer_view(
        &self,
    ) -> Result<typeyd3d12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(typeyd3d12::SVertexBufferView::create(
                self.raw.getgpuvirtualaddress(),
                (count * sizeofentry) as u32,
                sizeofentry as u32,
            ))
        } else {
            Err("Trying to create vertexbufferview for non-buffer resource")
        }
    }

    pub fn create_index_buffer_view(
        &self,
        format: typeyd3d12::EFormat,
    ) -> Result<typeyd3d12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(typeyd3d12::SIndexBufferView::create(
                self.raw.getgpuvirtualaddress(),
                format,
                (count * sizeofentry) as u32,
            ))
        } else {
            Err("Trying to create indexbufferview for non-buffer resource")
        }
    }

}

// ---------------------------------------------------------------------------------------------
// Swap chain functions
// ---------------------------------------------------------------------------------------------

impl SSwapChain {

    pub fn current_backbuffer_index(&self) -> usize {
        self.raw.currentbackbufferindex()
    }

    pub fn present(&mut self, sync_interval: u32, flags: u32) -> Result<(), &'static str> {
        self.raw.present(sync_interval, flags)
    }

    pub fn get_desc(&self) -> Result<typeyd3d12::SSwapChainDesc, &'static str> {
        self.raw.getdesc()
    }

    pub fn resize_buffers(
        &mut self,
        buffercount: u32,
        width: u32,
        height: u32,
        olddesc: &typeyd3d12::SSwapChainDesc,
    ) -> Result<(), &'static str> {
        self.raw.resizebuffers(buffercount, width, height, olddesc)
    }
}

fn update_subresources_stack(
    commandlist: &mut SCommandList,
    destinationresource: &mut SResource,
    intermediateresource: &mut SResource,
    intermediateoffset: u64,
    firstsubresource: u32,
    numsubresources: u32,
    srcdata: &mut typeyd3d12::SSubResourceData,
) {
    unsafe {
        directxgraphicssamples::UpdateSubresourcesStack(
            commandlist.raw.raw().as_raw(),
            destinationresource.raw.raw_mut().as_raw(),
            intermediateresource.raw.raw_mut().as_raw(),
            intermediateoffset,
            firstsubresource,
            numsubresources,
            srcdata.raw_mut(),
        );
    }
}


impl SCommandList {
    pub fn update_buffer_resource<T>(
        &mut self,
        device: &SDevice,
        bufferdata: &[T],
        flags: typeyd3d12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {

        unsafe {

            let mut destinationresource = device.create_committed_buffer_resource(
                typeyd3d12::EHeapType::Default,
                flags,
                typeyd3d12::EResourceStates::CopyDest,
                bufferdata
            )?;

            // -- resource created with Upload type MUST have state GenericRead
            let mut intermediateresource = device.create_committed_buffer_resource(
                typeyd3d12::EHeapType::Upload,
                flags,
                typeyd3d12::EResourceStates::GenericRead,
                bufferdata
            )?;

            let mut srcdata = typeyd3d12::SSubResourceData::createbuffer(bufferdata);
            update_subresources_stack(
                self,
                &mut destinationresource,
                &mut intermediateresource,
                0,
                0,
                1,
                &mut srcdata,
            );

            Ok(SCommandQueueUpdateBufferResult {
                destinationresource: destinationresource,
                intermediateresource: intermediateresource,
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
    pub fn signal_internal_fence(&mut self) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        self.signal(&self.fence, self.nextfencevalue)
    }

    pub fn flush_blocking(&mut self) -> Result<(), &'static str> {
        let lastfencevalue = self.signal_internal_fence()?;
        self.fence.wait_for_value(&mut self.fenceevent, lastfencevalue);
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
        swap_chain: &mut SSwapChain,
        descriptor_heap: &mut SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap_chain.backbuffers.is_empty());

        match descriptor_heap.type_() {
            typeyd3d12::EDescriptorHeapType::RenderTarget => {

                for backbuffidx in 0usize..2usize {
                    let rawresource = swap_chain.raw.getbuffer(backbuffidx)?;

                    let resource = SResource{
                        raw: rawresource,
                        metadata: EResourceMetadata::SwapChainResource,
                    };

                    swap_chain.backbuffers.push(resource);

                    let curdescriptorhandle = descriptor_heap.cpu_handle(backbuffidx)?;
                    self.create_render_target_view(
                        &mut swap_chain.backbuffers[backbuffidx],
                        &curdescriptorhandle,
                    )?;
                }

                Ok(())
            }
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap."),
        }
    }
}

impl SDescriptorHeap {
    pub fn cpu_handle(
        &self,
        index: usize,
    ) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.cpu_handle_heap_start();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

pub fn createsd3d12window(
    windowclass: &safewindows::SWindowClass,
    factory: &mut SFactory,
    device: &mut SDevice,
    commandqueue: &mut SCommandQueue,
    title: &str,
    width: u32,
    height: u32,
) -> Result<SD3D12Window, &'static str> {
    let window = rustywindows::SWindow::create(windowclass, title, width, height).unwrap(); // $$$FRK(TODO): this panics, need to unify error handling

    let swap_chain = factory.create_swap_chain(
        &window.raw(),
        commandqueue,
        width,
        height
    )?;
    let cur_buffer = swap_chain.current_backbuffer_index();

    let descriptor_heap = device.create_descriptor_heap(
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
    pub fn init_render_target_views(&mut self, device: &mut SDevice) -> Result<(), &'static str> {
        device.init_render_target_views(&mut self.swapchain, &mut self.rtvdescriptorheap)?;
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
    ) -> Result<typeyd3d12::SDescriptorHandle, &'static str> {
        self.rtvdescriptorheap.cpu_handle(self.curbuffer)
    }

    pub fn present(&mut self) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): figure out what this value does
        let syncinterval = 1;
        self.swapchain.present(syncinterval, 0)?;
        let newbuffer = self.swapchain.current_backbuffer_index();
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
        device: &mut SDevice,
    ) -> Result<(), &'static str> {
        if self.curwidth != width || self.curheight != height {
            let newwidth = std::cmp::max(1, width);
            let newheight = std::cmp::max(1, height);
            commandqueue.flush_blocking()?;

            self.swapchain.backbuffers.clear();

            let desc = self.swapchain.get_desc()?;
            self.swapchain.resize_buffers(
                2,
                newwidth,
                newheight,
                &desc,
            )?;

            self.curbuffer = self.swapchain.current_backbuffer_index();
            self.init_render_target_views(device)?;

            self.curwidth = newwidth;
            self.curheight = newheight;
        }

        Ok(())
    }
}
