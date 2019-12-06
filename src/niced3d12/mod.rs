#![allow(dead_code)]

mod window;
mod factory;
mod adapter;
mod device;
mod swapchain;
mod commandlist;

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

pub use self::factory::*;
pub use self::adapter::*;
pub use self::device::*;
pub use self::swapchain::*;
pub use self::commandlist::*;

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

// ---------------------------------------------------------------------------------------------
// Command allocator functions
// ---------------------------------------------------------------------------------------------
impl SCommandAllocator {
    pub fn reset(&mut self) {
        self.raw.reset();
    }
}

// ---------------------------------------------------------------------------------------------
// Command queue functions
// ---------------------------------------------------------------------------------------------

impl SCommandQueue {

    pub fn execute_command_list(
        &self, // -- verified thread safe in docs
        list: &mut SCommandList,
    ) -> Result<(), &'static str> {
        unsafe {
            list.raw().close()?;
            self.raw.executecommandlist(&list.raw())
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
        format: t12::EDXGIFormat,
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
            commandlist.raw().raw().as_raw(),
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


