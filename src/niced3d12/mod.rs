#![allow(dead_code)]

mod window;

use collections::{SPool, SPoolHandle};
use directxgraphicssamples;
use safewindows;
use typeyd3d12 as t12;

use std::cell::{RefCell};
use std::ops::{Deref};
use std::ptr;
use std::marker::{PhantomData};

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

// =================================================================================================
// MAIN TYPES
// =================================================================================================

pub struct SFactory {
    raw: t12::SFactory,
}

pub struct SAdapter {
    raw: t12::SAdapter4,
}

pub struct SDevice {
    raw: t12::SDevice,
}

pub struct SSwapChain {
    raw: t12::SSwapChain,

    pub buffercount: u32,
    pub backbuffers: Vec<SResource>, // $$FRK(TODO): VecArray PLEASE
}

pub struct SCommandList {
    raw: t12::SCommandList,
    //allocator: &RefCell<t12::SCommandAllocator>,
}

pub struct SCommandAllocator {
    raw: t12::SCommandAllocator,
}

pub struct SCommandQueue {
    raw: t12::SCommandQueue,

    fence: SFence,

    commandlisttype: t12::ECommandListType,
}

pub struct SFence {
    raw: t12::SFence,

    fenceevent: safewindows::SEventHandle,
    pub nextfencevalue: u64,
}

pub enum EResourceMetadata {
    Invalid,
    SwapChainResource,
    BufferResource { count: usize, sizeofentry: usize },
    Texture2DResource,
}

pub struct SResource {
    raw: t12::SResource,

    metadata: EResourceMetadata,
}

pub struct SDescriptorHeap {
    raw: t12::SDescriptorHeap,

    numdescriptors: u32,
    descriptorsize: usize,
    //cpudescriptorhandleforstart: t12::SDescriptorHandle<'heap, 'device>,
}

pub use self::window::SD3D12Window;

// =================================================================================================
// HELPER TYPES
// =================================================================================================

pub struct SCommandQueueUpdateBufferResult {
    pub destinationresource: SResource,
    pub intermediateresource: SResource,
}

struct SCommandListPoolList {
    list: SCommandList,
    allocator: SPoolHandle,
}

struct SCommandListPoolActiveAllocator {
    handle: SPoolHandle,
    reusefencevalue: u64,
}

pub struct SCommandListPool<'a> {
    queue: &'a RefCell<SCommandQueue>,

    allocators: SPool<SCommandAllocator>,
    lists: SPool<SCommandListPoolList>,

    activefence: SFence,
    activeallocators: Vec<SCommandListPoolActiveAllocator>,
}

// =================================================================================================
// IMPLS
// =================================================================================================

// -- $$FRK(TODO): almost every function in here should be unsafe
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
        &self,
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

                // -- $$$FRK(DNS): need a struct version of this in t12
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

    pub fn create_command_allocator(
        &self,
        type_: t12::ECommandListType,
    ) -> Result<SCommandAllocator, &'static str> {
        let raw = self.raw.createcommandallocator(type_)?;
        Ok(SCommandAllocator{ raw: raw })
    }

    // -- NOTE: This is unsafe because it initializes the list to an allocator which we don't
    // -- know is exclusive to the list
    pub unsafe fn create_command_list(
        &self,
        allocator: &mut SCommandAllocator,
    ) -> Result<SCommandList, &'static str> {
        let raw = self.raw.createcommandlist(&allocator.raw)?;
        Ok(SCommandList{ raw: raw })
    }

    pub fn create_fence(
        &self,
        winapi: &safewindows::SWinAPI,
    ) -> Result<SFence, &'static str> {

        let fence = self.raw.createfence()?;
        Ok(SFence {
            raw: fence,
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
        })
    }

    pub fn create_render_target_view(
        &self,
        render_target_resource: &mut SResource,
        dest_descriptor: &t12::SDescriptorHandle,
    ) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): assert on resource metadata
        self.raw.createrendertargetview(&render_target_resource.raw, dest_descriptor);
        Ok(())
    }

    pub fn create_depth_stencil_view(
        &self,
        depth_texture_resource: &mut SResource,
        desc: &t12::SDepthStencilViewDesc,
        dest_descriptor: t12::SDescriptorHandle,
    ) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): assert on resource metadata
        self.raw.create_depth_stencil_view(&depth_texture_resource.raw, desc, dest_descriptor);
        Ok(())
    }

    pub fn create_descriptor_heap(
        &self,
        type_: t12::EDescriptorHeapType,
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

    pub fn create_committed_texture2d_resource(
        &self, // verified thread safe via docs
        heap_type: t12::EHeapType,
        width: u32,
        height: u32,
        array_size: u16,
        mip_levels: u16,
        format: t12::EDXGIFormat,
        clear_value: t12::SClearValue,
        flags: t12::SResourceFlags,
        initial_resource_state: t12::EResourceStates,
    ) -> Result<SResource, &'static str> {

        let destinationresource = self.raw.createcommittedresource(
            t12::SHeapProperties::create(heap_type),
            t12::EHeapFlags::ENone,
            t12::SResourceDesc::create_texture_2d(width, height, array_size, mip_levels, format, flags),
            initial_resource_state,
            Some(clear_value),
        )?;

        Ok(SResource {
            raw: destinationresource,
            metadata: EResourceMetadata::Texture2DResource,
        })
    }

    pub fn create_committed_buffer_resource<T>(
        &self, // verified thread safe via docs
        heaptype: t12::EHeapType,
        flags: t12::SResourceFlags,
        initial_resource_state: t12::EResourceStates,
        bufferdata: &[T],
    ) -> Result<SResource, &'static str> {
        let buffersize = bufferdata.len() * std::mem::size_of::<T>();

        let destinationresource = self.raw.createcommittedresource(
            t12::SHeapProperties::create(heaptype),
            t12::EHeapFlags::ENone,
            t12::SResourceDesc::createbuffer(buffersize, flags),
            initial_resource_state,
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

    pub fn raw(&self) -> &t12::SDevice {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut t12::SDevice {
        &mut self.raw
    }
}

// ---------------------------------------------------------------------------------------------
// Command allocator functions
// ---------------------------------------------------------------------------------------------
impl SCommandAllocator {
    pub fn reset(&mut self) {
        self.raw.reset();
    }
}

// ---------------------------------------------------------------------------------------------
// Command List functions
// ---------------------------------------------------------------------------------------------
impl SCommandList {
    // -- by default, unsafe blocks are here because we are guaranteeing exclusive access to
    // -- the CommandList via the &mut self reference

    pub fn reset(&mut self, allocator: &mut SCommandAllocator) -> Result<(), &'static str> {
        unsafe { self.raw.reset(&allocator.raw) }
    }

    pub fn transition_resource(
        &mut self,
        resource: &SResource,
        beforestate: t12::EResourceStates,
        afterstate: t12::EResourceStates,
    ) -> Result<(), &'static str> {
        let transbarrier =
            t12::create_transition_barrier(&resource.raw, beforestate, afterstate);
        unsafe { self.raw.resourcebarrier(1, &[transbarrier]) };
        Ok(())
    }

    pub fn clear_render_target_view(
        &mut self,
        rtvdescriptor: t12::SDescriptorHandle,
        colour: &[f32; 4],
    ) -> Result<(), &'static str> {
        unsafe { self.raw.clearrendertargetview(rtvdescriptor, colour) };
        Ok(())
    }

    pub fn clear_depth_stencil_view(
        &mut self,
        dsv_descriptor: t12::SDescriptorHandle,
        depth: f32,
    ) -> Result<(), &'static str> {
        unsafe { self.raw.clear_depth_stencil_view(dsv_descriptor, depth) };
        Ok(())
    }

    pub fn set_pipeline_state(&mut self, pipeline_state: &t12::SPipelineState) {
        unsafe { self.raw.set_pipeline_state(pipeline_state) }
    }

    pub fn set_graphics_root_signature(&mut self, root_signature: &t12::SRootSignature) {
        unsafe { self.raw.set_graphics_root_signature(root_signature) }
    }

    pub fn ia_set_primitive_topology(&mut self, primitive_topology: t12::EPrimitiveTopology) {
        unsafe { self.raw.ia_set_primitive_topology(primitive_topology) }
    }

    pub fn ia_set_vertex_buffers(&mut self, start_slot: u32, vertex_buffers: &[&t12::SVertexBufferView]) {
        unsafe { self.raw.ia_set_vertex_buffers(start_slot, vertex_buffers) }
    }

    pub fn ia_set_index_buffer(&mut self, index_buffer: &t12::SIndexBufferView) {
        unsafe { self.raw.ia_set_index_buffer(index_buffer) }
    }

    pub fn rs_set_viewports(&mut self, viewports: &[&t12::SViewport]) {
        unsafe { self.raw.rs_set_viewports(viewports) }
    }

    pub fn rs_set_scissor_rects(&mut self, scissor_rects: t12::SScissorRects) {
        unsafe { self.raw.rs_set_scissor_rects(scissor_rects) }
    }

    pub fn om_set_render_targets(
        &self,
        render_target_descriptors: &[&t12::SDescriptorHandle],
        rts_single_handle_to_descriptor_range: bool,
        depth_target_descriptor: &t12::SDescriptorHandle) {

        unsafe { self.raw.om_set_render_targets(
            render_target_descriptors,
            rts_single_handle_to_descriptor_range,
            depth_target_descriptor
        )};
    }

    pub fn set_graphics_root_32_bit_constants<T: Sized>(
        &mut self,
        root_parameter_index: u32,
        data: &T,
        dest_offset_in_32_bit_values: u32,
    ) {
        unsafe { self.raw.set_graphics_root_32_bit_constants(
            root_parameter_index,
            data,
            dest_offset_in_32_bit_values,
        )};
    }

    pub fn draw_indexed_instanced(
        &mut self,
        index_count_per_instance: u32,
        instance_count: u32,
        start_index_location: u32,
        base_vertex_location: i32,
        start_instance_location: u32
    ) {
        unsafe { self.raw.draw_indexed_instanced(
            index_count_per_instance,
            instance_count,
            start_index_location,
            base_vertex_location,
            start_instance_location,
        ) };
    }

    pub fn get_type(&self) -> t12::ECommandListType {
        self.raw.gettype()
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        unsafe { self.raw.close() }
    }

    pub fn update_buffer_resource<T>(
        &mut self,
        device: &SDevice,
        bufferdata: &[T],
        flags: t12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {

        let mut destinationresource = device.create_committed_buffer_resource(
            t12::EHeapType::Default,
            flags,
            t12::EResourceStates::CopyDest,
            bufferdata
        )?;

        // -- resource created with Upload type MUST have state GenericRead
        let mut intermediateresource = device.create_committed_buffer_resource(
            t12::EHeapType::Upload,
            flags,
            t12::EResourceStates::GenericRead,
            bufferdata
        )?;

        let mut srcdata = t12::SSubResourceData::createbuffer(bufferdata);
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

// ---------------------------------------------------------------------------------------------
// Command queue functions
// ---------------------------------------------------------------------------------------------

impl SCommandQueue {

    pub fn create(
        device: &mut SDevice,
        winapi: &safewindows::SWinAPI,
        commandlisttype: t12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {

        let qresult = device.raw.createcommandqueue(commandlisttype)?;

        Ok(SCommandQueue {
            raw: qresult,
            fence: device.create_fence(winapi)?,
            commandlisttype: commandlisttype,
        })
    }

    pub fn execute_command_list(
        &self, // -- verified thread safe in docs
        list: &mut SCommandList,
    ) -> Result<(), &'static str> {
        unsafe {
            list.raw.close()?;
            self.raw.executecommandlist(&list.raw)
        };
        Ok(())
    }

    pub fn signal(
        &self, // -- I'm assuming this is safe
        fence: &mut SFence,
    ) -> Result<u64, &'static str> {
        let result = fence.nextfencevalue;
        self.raw.signal(&fence.raw, fence.nextfencevalue)?;
        fence.nextfencevalue += 1;
        Ok(result)
    }

    pub fn internal_fence_value(&self) -> u64 {
        self.fence.raw.getcompletedvalue()
    }

    pub fn signal_internal_fence(&mut self) -> Result<u64, &'static str> {
        let result = self.fence.nextfencevalue;
        self.raw.signal(&self.fence.raw, self.fence.nextfencevalue)?;
        self.fence.nextfencevalue += 1;
        Ok(result)
    }

    pub fn wait_for_internal_fence_value(&self, value: u64) {
        self.fence.wait_for_value(value);
    }

    pub fn flush_blocking(&mut self) -> Result<(), &'static str> {
        let lastfencevalue = self.signal_internal_fence()?;
        self.fence.wait_for_value(lastfencevalue);
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// Fence functions
// ---------------------------------------------------------------------------------------------
impl SFence {

    pub fn wait_for_value(
        &self,
        val: u64,
    ) {
        self.wait_for_value_duration(val, <u64>::max_value()).unwrap();
    }

    pub fn wait_for_value_duration(
        &self,
        val: u64,
        duration: u64,
    ) -> Result<(), &'static str> {
        if self.raw.getcompletedvalue() < val {
            self.raw.seteventoncompletion(val, &self.fenceevent)?;
            self.fenceevent.waitforsingleobject(duration);
        }

        Ok(())
    }

}

// ---------------------------------------------------------------------------------------------
// Descriptor Heap functions
// ---------------------------------------------------------------------------------------------

impl SDescriptorHeap {
    pub fn type_(&self) -> t12::EDescriptorHeapType {
        self.raw.type_
    }

    pub fn cpu_handle_heap_start(
        &self,
    ) -> t12::SDescriptorHandle {
        self.raw.getcpudescriptorhandleforheapstart()
    }
}

// ---------------------------------------------------------------------------------------------
// Resource functions
// ---------------------------------------------------------------------------------------------

impl SResource {

    pub fn create_vertex_buffer_view(
        &self,
    ) -> Result<t12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SVertexBufferView::create(
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
        format: t12::EFormat,
    ) -> Result<t12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SIndexBufferView::create(
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

    pub fn get_desc(&self) -> Result<t12::SSwapChainDesc, &'static str> {
        self.raw.getdesc()
    }

    pub fn resize_buffers(
        &mut self,
        buffercount: u32,
        width: u32,
        height: u32,
        olddesc: &t12::SSwapChainDesc,
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
    srcdata: &mut t12::SSubResourceData,
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


impl Default for EResourceMetadata {
    fn default() -> Self {
        EResourceMetadata::Invalid
    }
}

impl t12::SSubResourceData {
    pub fn createbuffer<T>(data: &[T]) -> Self {
        let buffersize = data.len() * std::mem::size_of::<T>();
        unsafe { Self::create(data.as_ptr(), buffersize, buffersize) }
    }
}

/*
pub struct SBufferResourceResult {
    destinationresource: t12::SResource,
    intermediateresource: t12::SResource,
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
            t12::EDescriptorHeapType::RenderTarget => {

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
    ) -> Result<t12::SDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.cpu_handle_heap_start();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamRootSignature<'a> {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: &'a winapi::um::d3d12::ID3D12RootSignature,
}

impl<'a> SPipelineStateStreamRootSignature<'a> {
    pub fn create(src: &'a t12::SRootSignature) -> Self {
        Self {
            type_: t12::EPipelineStateSubobjectType::RootSignature.d3dtype(),
            value: src.raw.deref(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamVertexShader<'a> {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::um::d3d12::D3D12_SHADER_BYTECODE,
    phantom: PhantomData<&'a t12::SShaderBytecode<'a>>,
}

impl<'a> SPipelineStateStreamVertexShader<'a> {
    pub fn create(shader_bytecode: &'a t12::SShaderBytecode) -> Self {
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::VS.d3dtype(),
            value: unsafe { shader_bytecode.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamPixelShader<'a> {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::um::d3d12::D3D12_SHADER_BYTECODE,
    phantom: PhantomData<&'a t12::SShaderBytecode<'a>>,
}

impl<'a> SPipelineStateStreamPixelShader<'a> {
    pub fn create(shader_bytecode: &'a t12::SShaderBytecode) -> Self {
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::PS.d3dtype(),
            value: unsafe { shader_bytecode.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamInputLayout<'a> {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::um::d3d12::D3D12_INPUT_LAYOUT_DESC,
    phantom: PhantomData<&'a t12::SInputLayoutDesc>,
}

impl<'a> SPipelineStateStreamInputLayout<'a> {
    pub fn create(input_layout: &'a mut t12::SInputLayoutDesc) -> Self {
        Self {
            type_: t12::EPipelineStateSubobjectType::InputLayout.d3dtype(),
            value: unsafe { input_layout.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamPrimitiveTopology {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::um::d3d12::D3D12_PRIMITIVE_TOPOLOGY_TYPE,
}

impl SPipelineStateStreamPrimitiveTopology {
    pub fn create(value: t12::EPrimitiveTopologyType) -> Self {
        Self {
            type_: t12::EPipelineStateSubobjectType::PrimitiveTopology.d3dtype(),
            value: value.d3dtype(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamRTVFormats<'a> {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::um::d3d12::D3D12_RT_FORMAT_ARRAY,
    phantom: PhantomData<&'a t12::SRTFormatArray>,
}

impl<'a> SPipelineStateStreamRTVFormats<'a> {
    pub fn create(format_array: &t12::SRTFormatArray) -> Self {
        Self {
            type_: t12::EPipelineStateSubobjectType::RenderTargetFormats.d3dtype(),
            value: format_array.d3dtype(),
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamDepthStencilFormat {
    type_: winapi::um::d3d12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: winapi::shared::dxgiformat::DXGI_FORMAT,
}

impl SPipelineStateStreamDepthStencilFormat {
    pub fn create(format: t12::EDXGIFormat) -> Self {
        Self {
            type_: t12::EPipelineStateSubobjectType::DepthStencilFormat.d3dtype(),
            value: format.d3dtype(),
        }
    }
}

impl<'a> SCommandListPool<'a> {
    pub fn create(
        device: &SDevice,
        queue: &'a RefCell<SCommandQueue>,
        winapi: &safewindows::SWinAPI,
        num_lists: u16,
        num_allocators: u16) -> Result<Self, &'static str> {

        assert!(num_allocators > 0 && num_lists > 0);

        let type_ = queue.borrow().commandlisttype;

        let mut allocators = Vec::new();
        let mut lists = Vec::new();

        for _ in 0..num_allocators {
            allocators.push(device.create_command_allocator(type_)?);
        }

        for _ in 0..num_lists {
            let mut list = unsafe { device.create_command_list(&mut allocators[0])? } ;
            // -- immediately close handle because we'll re-assign a new allocator from the pool when ready
            list.close()?;
            lists.push(SCommandListPoolList{
                list: list,
                allocator: Default::default(),
            });
        }

        Ok(Self {
            queue: queue,
            allocators: SPool::<SCommandAllocator>::create_from_vec(0, num_allocators, allocators),
            lists: SPool::<SCommandListPoolList>::create_from_vec(1, num_lists, lists),
            activefence: device.create_fence(winapi)?,
            activeallocators: Vec::<SCommandListPoolActiveAllocator>::with_capacity(num_allocators as usize),
        })
    }

    fn free_allocators(&mut self) {
        let completedvalue = self.queue.borrow().internal_fence_value();
        for alloc in &self.activeallocators {
            if alloc.reusefencevalue <= completedvalue {
                self.allocators.free(alloc.handle);
            }
        }

        self.activeallocators
            .retain(|alloc| alloc.reusefencevalue > completedvalue);
    }

    pub fn alloc_list(&mut self) -> Result<SPoolHandle, &'static str> {
        self.free_allocators();

        if self.lists.full() || self.allocators.full() {
            return Err("no available command list or allocator");
        }

        let allocatorhandle = self.allocators.alloc()?;
        let allocator = self.allocators.get_mut(allocatorhandle)?;
        allocator.reset();

        let listhandle = self.lists.alloc()?;
        let list = self.lists.get_mut(listhandle)?;
        list.list.reset(allocator)?;
        list.allocator = allocatorhandle;

        Ok(listhandle)
    }

    pub fn get_list(&mut self, handle: SPoolHandle) -> Result<&mut SCommandList, &'static str> {
        let list = self.lists.get_mut(handle)?;
        Ok(&mut list.list)
    }

    pub fn execute_and_free_list(&mut self, handle: SPoolHandle) -> Result<u64, &'static str> {
        let allocator = {
            let list = self.lists.get_mut(handle)?;
            assert!(list.list.get_type() == self.queue.borrow().commandlisttype);
            self.queue.borrow().execute_command_list(&mut list.list)?;

            assert!(list.allocator.valid());
            list.allocator
        };
        self.lists.free(handle);

        let fenceval = self.queue.borrow().signal(&mut self.activefence)?;

        self.activeallocators.push(SCommandListPoolActiveAllocator {
            handle: allocator,
            reusefencevalue: fenceval,
        });

        Ok(fenceval)
    }

    pub fn wait_for_internal_fence_value(&self, value: u64) {
        self.activefence.wait_for_value(value);
    }

    pub fn flush_blocking(&mut self) -> Result<(), &'static str> {
        self.queue.borrow_mut().flush_blocking()
    }
}

