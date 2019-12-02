#![allow(dead_code)]

use safewindows;

use std::{mem, ptr};

use arrayvec::{ArrayVec};

use winapi::ctypes::c_void;
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::dxgi1_4::*;
use winapi::shared::dxgi1_5::*;
use winapi::shared::dxgi1_6::*;
use winapi::shared::minwindef::*;
use winapi::shared::{dxgiformat, dxgitype, winerror};
use winapi::um::d3d12::*;
use winapi::um::d3d12sdklayers::*;
use winapi::um::{d3dcommon, d3dcompiler, unknwnbase};
use winapi::Interface;

use wio::com::ComPtr;

// -- this is copied in safewindows, does it have to be?
trait ComPtrPtrs<T> {
    unsafe fn asunknownptr(&self) -> *mut unknwnbase::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T>
where
    T: Interface,
{
    unsafe fn asunknownptr(&self) -> *mut unknwnbase::IUnknown {
        self.as_raw() as *mut unknwnbase::IUnknown
    }
}

macro_rules! returnerrifwinerror {
    ($hn:expr, $err:expr) => {
        if !winerror::SUCCEEDED($hn) {
            return Err($err);
        }
    };
}

pub struct SDebugInterface {
    debuginterface: ComPtr<ID3D12Debug>,
}

pub fn getdebuginterface() -> Result<SDebugInterface, &'static str> {
    unsafe {
        let mut result: SDebugInterface = mem::uninitialized();

        let riid = ID3D12Debug::uuidof();
        let voidcasted: *mut *mut c_void = &mut result.debuginterface as *mut _ as *mut *mut c_void;

        let hresult = D3D12GetDebugInterface(&riid, voidcasted);
        if winerror::SUCCEEDED(hresult) {
            Ok(result)
        } else {
            Err("D3D12GetDebugInterface gave an error.")
        }
    }
}

impl SDebugInterface {
    pub fn enabledebuglayer(&self) -> () {
        unsafe {
            self.debuginterface.EnableDebugLayer();
        }
    }
}

pub struct SFactory {
    factory: ComPtr<IDXGIFactory4>,
}

pub fn createdxgifactory4() -> Result<SFactory, &'static str> {
    let mut rawfactory: *mut IDXGIFactory4 = ptr::null_mut();
    let createfactoryresult = unsafe {
        CreateDXGIFactory2(
            DXGI_CREATE_FACTORY_DEBUG,
            &IDXGIFactory4::uuidof(),
            &mut rawfactory as *mut *mut _ as *mut *mut c_void,
        )
    };
    if winerror::SUCCEEDED(createfactoryresult) {
        return Ok(SFactory {
            factory: unsafe { ComPtr::from_raw(rawfactory) },
        });
    }

    Err("Couldn't get D3D12 factory.")
}

pub struct SAdapter1 {
    adapter: ComPtr<IDXGIAdapter1>,
}

impl SAdapter1 {
    pub fn getdesc(&self) -> DXGI_ADAPTER_DESC1 {
        let mut adapterdesc: DXGI_ADAPTER_DESC1 = unsafe { mem::uninitialized() };
        unsafe { self.adapter.GetDesc1(&mut adapterdesc) };
        return adapterdesc;
    }

    pub fn castadapter4(&self) -> Option<SAdapter4> {
        match self.adapter.cast::<IDXGIAdapter4>() {
            Ok(a) => {
                return Some(SAdapter4 { adapter: a });
            }
            Err(_) => {
                return None;
            }
        };
    }

    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(self.adapter.asunknownptr())
    }
}

#[derive(Clone)]
pub struct SAdapter4 {
    adapter: ComPtr<IDXGIAdapter4>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EResourceStates {
    Common,
    VertexAndConstantBuffer,
    IndexBuffer,
    RenderTarget,
    UnorderedAccess,
    DepthWrite,
    DepthRead,
    NonPixelShaderResource,
    PixelShaderResource,
    StreamOut,
    IndirectArgument,
    CopyDest,
    CopySource,
    ResolveDest,
    ResolveSource,
    GenericRead,
    Present,
    Predication,
}

impl EResourceStates {
    fn d3dtype(&self) -> D3D12_RESOURCE_STATES {
        match self {
            EResourceStates::Common => D3D12_RESOURCE_STATE_COMMON,
            EResourceStates::VertexAndConstantBuffer => {
                D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER
            }
            EResourceStates::IndexBuffer => D3D12_RESOURCE_STATE_INDEX_BUFFER,
            EResourceStates::RenderTarget => D3D12_RESOURCE_STATE_RENDER_TARGET,
            EResourceStates::UnorderedAccess => D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            EResourceStates::DepthWrite => D3D12_RESOURCE_STATE_DEPTH_WRITE,
            EResourceStates::DepthRead => D3D12_RESOURCE_STATE_DEPTH_READ,
            EResourceStates::NonPixelShaderResource => {
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE
            }
            EResourceStates::PixelShaderResource => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            EResourceStates::StreamOut => D3D12_RESOURCE_STATE_STREAM_OUT,
            EResourceStates::IndirectArgument => D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            EResourceStates::CopyDest => D3D12_RESOURCE_STATE_COPY_DEST,
            EResourceStates::CopySource => D3D12_RESOURCE_STATE_COPY_SOURCE,
            EResourceStates::ResolveDest => D3D12_RESOURCE_STATE_RESOLVE_DEST,
            EResourceStates::ResolveSource => D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
            EResourceStates::GenericRead => D3D12_RESOURCE_STATE_GENERIC_READ,
            EResourceStates::Present => D3D12_RESOURCE_STATE_PRESENT,
            EResourceStates::Predication => D3D12_RESOURCE_STATE_PREDICATION,
        }
    }
}

pub struct SBarrier {
    barrier: D3D12_RESOURCE_BARRIER,
}

impl SFactory {
    pub fn enumadapters(&self, adapteridx: u32) -> Option<SAdapter1> {
        let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();

        if unsafe { self.factory.EnumAdapters1(adapteridx, &mut rawadapter1) }
            == winerror::DXGI_ERROR_NOT_FOUND
        {
            return None;
        }

        let adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };
        Some(SAdapter1 { adapter: adapter1 })
    }
}

pub fn createtransitionbarrier(
    resource: &SResource,
    beforestate: EResourceStates,
    afterstate: EResourceStates,
) -> SBarrier {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        u: unsafe { mem::zeroed() },
    };

    *unsafe { barrier.u.Transition_mut() } = D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: resource.resource.as_raw(),
        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: beforestate.d3dtype(),
        StateAfter: afterstate.d3dtype(),
    };

    SBarrier { barrier: barrier }
}

#[derive(Clone)]
pub struct SDevice {
    device: ComPtr<ID3D12Device2>,
}

impl SDevice {
    pub fn castinfoqueue(&self) -> Option<SInfoQueue> {
        match self.device.cast::<ID3D12InfoQueue>() {
            Ok(a) => {
                return Some(SInfoQueue { infoqueue: a });
            }
            Err(_) => {
                return None;
            }
        };
    }

    pub fn createcommandqueue(
        &self,
        type_: ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {
        // -- $$$FRK(TODO): pass priority, flags, nodemask
        let desc = D3D12_COMMAND_QUEUE_DESC {
            Type: type_.d3dtype(),
            Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32,
            Flags: 0,
            NodeMask: 0,
        };

        let mut rawqueue: *mut ID3D12CommandQueue = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateCommandQueue(
                &desc,
                &ID3D12CommandQueue::uuidof(),
                &mut rawqueue as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hr, "Could not create command queue");

        Ok(SCommandQueue {
            queue: unsafe { ComPtr::from_raw(rawqueue) },
        })
    }

    pub fn create_descriptor_heap(
        &self,
        type_: EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        let desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: type_.d3dtype(),
            NumDescriptors: numdescriptors,
            Flags: 0,
            NodeMask: 0,
        };

        let mut rawheap: *mut ID3D12DescriptorHeap = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(
                &desc,
                &ID3D12DescriptorHeap::uuidof(),
                &mut rawheap as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hr, "Failed to create descriptor heap");

        let heap = unsafe { ComPtr::from_raw(rawheap) };

        Ok(SDescriptorHeap {
            type_: type_,
            heap: heap,
        })
    }

    pub fn getdescriptorhandleincrementsize(&self, type_: EDescriptorHeapType) -> usize {
        unsafe {
            self.device
                .GetDescriptorHandleIncrementSize(type_.d3dtype()) as usize
        }
    }

    // -- $$$FRK(TODO): allow pDesc parameter
    pub fn createrendertargetview(&self, resource: &SResource, destdescriptor: &SDescriptorHandle) {
        unsafe {
            self.device.CreateRenderTargetView(
                resource.resource.as_raw(),
                ptr::null(),
                destdescriptor.handle,
            );
        }
    }

    pub fn create_depth_stencil_view(
        &self,
        resource: &SResource,
        desc: &SDepthStencilViewDesc,
        dest_descriptor: SDescriptorHandle,
    ) {
        unsafe {
            let d3ddesc = desc.d3dtype();

            self.device.CreateDepthStencilView(
                resource.resource.as_raw(),
                &d3ddesc,
                dest_descriptor.handle,
            );
        }
    }

    // -- $$$FRK(TODO): Wrapper for D3D12 Resource Flags?
    pub fn createcommittedresource(
        &self,
        heapproperties: SHeapProperties,
        heapflags: EHeapFlags,
        resourcedesc: SResourceDesc,
        initialresourcestate: EResourceStates,
        clear_value: Option<SClearValue>,
    ) -> Result<SResource, &'static str> {
        unsafe {
            #[allow(unused_assignments)]
            let mut d3dcv : D3D12_CLEAR_VALUE = mem::uninitialized();

            let clear_value_ptr : * const D3D12_CLEAR_VALUE = match clear_value {
                Some(cv) => {
                    d3dcv = cv.d3dtype();
                    &d3dcv
                },
                None => ptr::null_mut(),
            };

            let mut rawresource: *mut ID3D12Resource = ptr::null_mut();
            let hn = self.device.CreateCommittedResource(
                &heapproperties.raw,
                heapflags.d3dtype(),
                &resourcedesc.raw,
                initialresourcestate.d3dtype(),
                clear_value_ptr,
                &ID3D12Resource::uuidof(), // $$$FRK(TODO): this isn't necessarily right
                &mut rawresource as *mut *mut _ as *mut *mut c_void,
            );

            returnerrifwinerror!(hn, "Could not create committed resource.");
            Ok(SResource {
                resource: ComPtr::from_raw(rawresource),
            })
        }
    }

    pub fn createcommandallocator(
        &self,
        type_: ECommandListType,
    ) -> Result<SCommandAllocator, &'static str> {
        let mut rawca: *mut ID3D12CommandAllocator = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandAllocator(
                type_.d3dtype(),
                &ID3D12CommandAllocator::uuidof(),
                &mut rawca as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create command allocator.");

        Ok(SCommandAllocator {
            type_: type_,
            commandallocator: unsafe { ComPtr::from_raw(rawca) },
        })
    }

    pub fn createcommandlist(
        &self,
        allocator: &SCommandAllocator,
    ) -> Result<SCommandList, &'static str> {
        let mut rawcl: *mut ID3D12GraphicsCommandList = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandList(
                0,
                allocator.type_.d3dtype(),
                allocator.commandallocator.as_raw(),
                ptr::null_mut(),
                &ID3D12GraphicsCommandList::uuidof(),
                &mut rawcl as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create command list.");

        Ok(SCommandList {
            commandlist: unsafe { ComPtr::from_raw(rawcl) },
        })
    }

    // -- $$$FRK(TODO): think about mutable refs for lots of fns here and in safewindows
    pub fn createfence(&self) -> Result<SFence, &'static str> {
        let mut rawf: *mut ID3D12Fence = ptr::null_mut();
        let hn = unsafe {
            // -- $$$FRK(TODO): support parameters
            self.device.CreateFence(
                0,
                D3D12_FENCE_FLAG_NONE,
                &ID3D12Fence::uuidof(),
                &mut rawf as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create fence.");

        Ok(SFence {
            fence: unsafe { ComPtr::from_raw(rawf) },
        })
    }

    // -- $$$FRK(TODO): support nodeMask parameter
    pub fn create_root_signature(&self, blob_with_root_signature: &SBlob)-> Result<SRootSignature, &'static str> {

        let mut raw_root_signature : *mut ID3D12RootSignature = ptr::null_mut();

        let hr = unsafe { self.device.CreateRootSignature(
            0,
            blob_with_root_signature.raw.GetBufferPointer(),
            blob_with_root_signature.raw.GetBufferSize(),
            &ID3D12RootSignature::uuidof(),
            &mut raw_root_signature as *mut *mut _ as *mut *mut c_void,
        )};
        returnerrifwinerror!(hr, "Could not create root signature");

        let root_signature = unsafe { ComPtr::from_raw(raw_root_signature) };
        Ok(SRootSignature{
            raw: root_signature,
        })
    }

    pub fn create_pipeline_state_for_raw_desc(&self, desc: &D3D12_PIPELINE_STATE_STREAM_DESC) -> Result<SPipelineState, &'static str> {
        let mut raw_pipeline_state : *mut ID3D12PipelineState = ptr::null_mut();

        let hr = unsafe { self.device.CreatePipelineState(
            desc,
            &ID3D12PipelineState::uuidof(),
            &mut raw_pipeline_state as *mut *mut _ as *mut *mut c_void,
        )};
        returnerrifwinerror!(hr, "Could not create pipeline state");

        let pipeline_state = unsafe { ComPtr::from_raw(raw_pipeline_state) };
        Ok(SPipelineState{
            raw: pipeline_state,
        })
    }

    pub fn create_pipeline_state<T>(&self, desc: &SPipelineStateStreamDesc<T>) -> Result<SPipelineState, &'static str> {
        let d3ddesc = unsafe { desc.d3dtype() };
        self.create_pipeline_state_for_raw_desc(&d3ddesc)
    }
}

pub struct SInfoQueue {
    infoqueue: ComPtr<ID3D12InfoQueue>,
}

impl SInfoQueue {
    pub fn setbreakonseverity(&self, id: D3D12_MESSAGE_ID, val: BOOL) {
        unsafe {
            self.infoqueue.SetBreakOnSeverity(id, val);
        }
    }

    pub fn pushstoragefilter(
        &self,
        filter: &mut D3D12_INFO_QUEUE_FILTER,
    ) -> Result<(), &'static str> {
        let hn = unsafe { self.infoqueue.PushStorageFilter(filter) };
        returnerrifwinerror!(hn, "Could not push storage filter on infoqueue.");
        Ok(())
    }
}

fn d3d12createdevice(adapter: *mut unknwnbase::IUnknown) -> Result<SDevice, &'static str> {
    let mut rawdevice: *mut ID3D12Device2 = ptr::null_mut();
    let hn = unsafe {
        D3D12CreateDevice(
            adapter, //self.adapter.asunknownptr(),
            d3dcommon::D3D_FEATURE_LEVEL_11_0,
            &ID3D12Device2::uuidof(),
            &mut rawdevice as *mut *mut _ as *mut *mut c_void,
        )
    };
    returnerrifwinerror!(hn, "Could not create device on adapter.");

    let device = unsafe { ComPtr::from_raw(rawdevice) };
    Ok(SDevice { device: device })
}

impl SAdapter4 {
    pub unsafe fn d3d12createdevice(&self) -> Result<SDevice, &'static str> {
        d3d12createdevice(self.adapter.asunknownptr())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ECommandListType {
    Invalid,
    Direct,
    Bundle,
    Compute,
    Copy,
    //VideoDecode,
    //VideoProcess,
}

impl ECommandListType {
    fn d3dtype(&self) -> D3D12_COMMAND_LIST_TYPE {
        match self {
            ECommandListType::Invalid => D3D12_COMMAND_LIST_TYPE_DIRECT, // $$$FRK(TODO): obviously wrong, this needs to return an option I guess
            ECommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
            ECommandListType::Bundle => D3D12_COMMAND_LIST_TYPE_BUNDLE,
            ECommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            ECommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
            //VideoDecode => D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE ,
            //VideoProcess => D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS ,
        }
    }

    fn create(d3dtype: D3D12_COMMAND_LIST_TYPE) -> Self {
        match d3dtype {
            D3D12_COMMAND_LIST_TYPE_DIRECT => ECommandListType::Direct,
            D3D12_COMMAND_LIST_TYPE_BUNDLE => ECommandListType::Bundle,
            D3D12_COMMAND_LIST_TYPE_COMPUTE => ECommandListType::Compute,
            D3D12_COMMAND_LIST_TYPE_COPY => ECommandListType::Copy,
            _ => ECommandListType::Invalid,
        }
    }
}

pub struct SCommandQueueDesc {
    raw: D3D12_COMMAND_QUEUE_DESC,
}

impl SCommandQueueDesc {
    pub fn cqtype(&self) -> ECommandListType {
        ECommandListType::create(self.raw.Type)
    }
}

#[derive(Clone)]
pub struct SCommandQueue {
    queue: ComPtr<ID3D12CommandQueue>,
}

#[derive(Clone)]
pub struct SResource {
    resource: ComPtr<ID3D12Resource>,
}

impl std::cmp::PartialEq for SResource {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
    }
}

impl SResource {
    pub unsafe fn raw_mut(&mut self) -> &mut ComPtr<ID3D12Resource> {
        &mut self.resource
    }

    pub fn getgpuvirtualaddress(&self) -> SGPUVirtualAddress {
        unsafe {
            SGPUVirtualAddress {
                raw: self.resource.GetGPUVirtualAddress(),
            }
        }
    }
}

#[derive(Clone)]
pub struct SSwapChain {
    swapchain: ComPtr<IDXGISwapChain4>,
}

impl SSwapChain {
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

        Ok(SResource {
            resource: unsafe { ComPtr::from_raw(rawbuf) },
        })
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

impl SFactory {
    pub unsafe fn createswapchainforwindow(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &SCommandQueue,
        width: u32,
        height: u32,
    ) -> Result<SSwapChain, &'static str> {
        let buffercount = 2;

        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM, // $$$FRK(TODO): I have no idea why I'm picking this format
            Stereo: FALSE,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            }, // $$$FRK(TODO): ???
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: buffercount,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
        };
        let mut rawswapchain: *mut IDXGISwapChain1 = ptr::null_mut();

        let hr = self.factory.CreateSwapChainForHwnd(
            commandqueue.queue.asunknownptr(),
            window.raw(),
            &desc,
            ptr::null(),
            ptr::null_mut(),
            &mut rawswapchain as *mut *mut _ as *mut *mut IDXGISwapChain1,
        );

        returnerrifwinerror!(hr, "Failed to create swap chain");

        let swapchain = ComPtr::from_raw(rawswapchain);

        match swapchain.cast::<IDXGISwapChain4>() {
            Ok(sc4) => Ok(SSwapChain { swapchain: sc4 }),
            _ => Err("Swap chain could not be case to SwapChain4"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum EDescriptorHeapType {
    ConstantBufferShaderResourceUnorderedAccess,
    Sampler,
    RenderTarget,
    DepthStencil,
}

impl EDescriptorHeapType {
    pub fn d3dtype(&self) -> u32 {
        match self {
            EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess => {
                D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV
            }
            EDescriptorHeapType::Sampler => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            EDescriptorHeapType::RenderTarget => D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            EDescriptorHeapType::DepthStencil => D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        }
    }
}

#[derive(Clone)]
pub struct SDescriptorHeap {
    pub type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
}

impl SDescriptorHeap {
    pub fn getcpudescriptorhandleforheapstart(&self) -> SDescriptorHandle {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SDescriptorHandle { handle: start }
    }
}

pub struct SDescriptorHandle {
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl SDescriptorHandle {
    pub unsafe fn offset(&self, bytes: usize) -> SDescriptorHandle {
        SDescriptorHandle {
            handle: D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + bytes,
            },
        }
    }
}

// -- $$$FRK(TODO): combine impls

#[derive(Copy, Clone, PartialEq)]
pub enum EHeapType {
    Default,
    Upload,
}

impl EHeapType {
    pub fn d3dtype(&self) -> D3D12_HEAP_TYPE {
        match self {
            EHeapType::Default => D3D12_HEAP_TYPE_DEFAULT,
            EHeapType::Upload => D3D12_HEAP_TYPE_UPLOAD,
        }
    }
}

pub struct SHeapProperties {
    raw: D3D12_HEAP_PROPERTIES,
}

impl SHeapProperties {
    pub fn create(type_: EHeapType) -> Self {
        Self {
            raw: D3D12_HEAP_PROPERTIES {
                Type: type_.d3dtype(),
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EHeapFlags {
    ENone,
}

impl TD3DFlags32 for EHeapFlags {
    type TD3DType = D3D12_HEAP_FLAGS;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            EHeapFlags::ENone => D3D12_HEAP_FLAG_NONE,
        }
    }
}
pub struct SResourceDesc {
    raw: D3D12_RESOURCE_DESC,
}

pub trait TD3DFlags32 {
    type TD3DType: std::convert::Into<u32> + std::convert::From<u32> + Copy + Clone;

    fn d3dtype(&self) -> Self::TD3DType;
}

/*
pub trait TConvertToD3DType {
    type TD3DType;

    fn d3dtype(&self) -> Self::TD3DType;
}
*/

pub struct SD3DFlags32<T: TD3DFlags32 + Copy> {
    raw: T::TD3DType,
}

impl<T: TD3DFlags32 + Copy> From<T> for SD3DFlags32<T> {
    fn from(flag: T) -> Self {
        Self::none().or(flag)
    }
}

impl<T: TD3DFlags32 + Copy> Clone for SD3DFlags32<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TD3DFlags32 + Copy> Copy for SD3DFlags32<T> {}

impl<T: TD3DFlags32 + Copy> SD3DFlags32<T> {
    pub fn none() -> Self {
        Self {
            raw: T::TD3DType::from(0),
        }
    }

    pub fn all() -> Self {
        Self {
            raw: T::TD3DType::from(std::u32::MAX),
        }
    }

    pub fn create(flags: &[T]) -> Self {
        let mut result = Self::none();
        for flag in flags {
            result = result.or(*flag);
        }
        result
    }

    pub fn and(&self, other: T) -> Self {
        let a: u32 = self.raw.into();
        let b: u32 = other.d3dtype().into();
        let res: u32 = a & b;
        Self {
            raw: T::TD3DType::from(res),
        }
    }

    pub fn or(&self, other: T) -> Self {
        let a: u32 = self.raw.into();
        let b: u32 = other.d3dtype().into();
        let res: u32 = a | b;
        Self {
            raw: T::TD3DType::from(res),
        }
    }

    pub fn d3dtype(&self) -> T::TD3DType {
        self.raw
    }
}

// -- $$$FRK(TODO): does not follow the philosophy of this file for creating rustic types for each
// -- D3D type. Furthermore, the helper methods belong in niced3d12
impl SResourceDesc {
    pub fn createbuffer(buffersize: usize, flags: SResourceFlags) -> Self {
        Self {
            raw: D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: buffersize as u64, // seems like this is used as the main dimension for a 1D resource
                Height: 1,                // required
                DepthOrArraySize: 1,      // required
                MipLevels: 1,             // required
                Format: dxgiformat::DXGI_FORMAT_UNKNOWN, // required
                SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR, // required
                Flags: flags.d3dtype(),
            },
        }
    }

    pub fn create_texture_2d(width: u32, height: u32, array_size: u16, mip_levels: u16, format: EDXGIFormat, flags: SResourceFlags) -> Self {
        Self {
            raw: D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: width as u64,
                Height: height,                // required
                DepthOrArraySize: array_size,      // required
                MipLevels: mip_levels,             // required
                Format: format.d3dtype(), // required
                SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN, // required
                Flags: flags.d3dtype(),
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EResourceFlags {
    ENone,
    AllowRenderTarget,
    AllowDepthStencil,
    AllowUnorderedAccess,
    DenyShaderResource,
    AllowCrossAdapter,
    AllowSimultaneousAccess,
}

impl TD3DFlags32 for EResourceFlags {
    type TD3DType = D3D12_HEAP_FLAGS;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            EResourceFlags::ENone => D3D12_RESOURCE_FLAG_NONE,
            EResourceFlags::AllowRenderTarget => D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            EResourceFlags::AllowDepthStencil => D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            EResourceFlags::AllowUnorderedAccess => D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            EResourceFlags::DenyShaderResource => D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
            EResourceFlags::AllowCrossAdapter => D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER,
            EResourceFlags::AllowSimultaneousAccess => {
                D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS
            }
        }
    }
}

pub type SResourceFlags = SD3DFlags32<EResourceFlags>;

#[derive(Clone)]
pub struct SCommandAllocator {
    type_: ECommandListType,
    commandallocator: ComPtr<ID3D12CommandAllocator>,
}

impl SCommandAllocator {
    pub fn reset(&self) {
        unsafe { self.commandallocator.Reset() };
    }
}

pub struct SScissorRects {
    rects: ArrayVec<[SRect; 16]>,

    d3drects: ArrayVec<[D3D12_RECT; 16]>,
}

impl SScissorRects {
    pub fn create(rects: &[&SRect]) -> Self {
        let mut result = Self {
            rects: ArrayVec::new(),
            d3drects: ArrayVec::new(),
        };

        for rect in rects {
            result.rects.push(**rect);
            result.d3drects.push(rect.d3dtype());
        }

        result
    }
}

#[derive(Clone)]
pub struct SCommandList {
    commandlist: ComPtr<ID3D12GraphicsCommandList>,
}

impl SCommandList {
    // -- almost everything in here is unsafe because we take shared references, but require
    // -- exclusive access to be thread safe

    pub fn gettype(&self) -> ECommandListType {
        unsafe { ECommandListType::create(self.commandlist.GetType()) }
    }

    pub unsafe fn reset(&self, commandallocator: &SCommandAllocator) -> Result<(), &'static str> {
        let hn = self.commandlist
            .Reset(commandallocator.commandallocator.as_raw(), ptr::null_mut());
        returnerrifwinerror!(hn, "Could not reset command list.");
        Ok(())
    }

    pub unsafe fn resourcebarrier(&self, numbarriers: u32, barriers: &[SBarrier]) {
        // -- $$$FRK(TODO): need to figure out how to make a c array from the rust slice
        // -- w/o a heap allocation...
        assert!(numbarriers == 1);
        self.commandlist.ResourceBarrier(1, &(barriers[0].barrier));
    }

    pub unsafe fn clearrendertargetview(&self, descriptor: SDescriptorHandle, colour: &[f32; 4]) {
        // -- $$$FRK(TODO): support third/fourth parameter
        self.commandlist
            .ClearRenderTargetView(descriptor.handle, colour, 0, ptr::null());
    }

    pub unsafe fn clear_depth_stencil_view(&self, descriptor: SDescriptorHandle, depth: f32) {
        // -- $$$FRK(TODO): support ClearFlags/Stencil/NumRects/pRects
        self.commandlist.ClearDepthStencilView(
            descriptor.handle, D3D12_CLEAR_FLAG_DEPTH, depth, 0, 0, ptr::null());
    }

    pub unsafe fn set_pipeline_state(&self, pipeline_state: &SPipelineState) {
        self.commandlist.SetPipelineState(pipeline_state.raw.as_raw())
    }

    pub unsafe fn set_graphics_root_signature(&self, root_signature: &SRootSignature) {
        self.commandlist.SetGraphicsRootSignature(root_signature.raw.as_raw())
    }

    pub unsafe fn ia_set_primitive_topology(&self, primitive_topology: EPrimitiveTopology) {
        self.commandlist.IASetPrimitiveTopology(primitive_topology.d3dtype())
    }

    pub unsafe fn ia_set_vertex_buffers(&self, start_slot: u32, vertex_buffers: &[&SVertexBufferView]) {
        assert!(vertex_buffers.len() == 1); // didn't want to implement copying d3dtype array
        self.commandlist.IASetVertexBuffers(start_slot, vertex_buffers.len() as u32, &vertex_buffers[0].raw)
    }

    pub unsafe fn ia_set_index_buffer(&self, index_buffer: &SIndexBufferView) {
        self.commandlist.IASetIndexBuffer(&index_buffer.raw)
    }

    pub unsafe fn rs_set_viewports(&self, viewports: &[&SViewport]) {
        assert!(viewports.len() == 1); // didn't want to implement copying d3dtype array
        self.commandlist.RSSetViewports(viewports.len() as u32, &viewports[0].viewport)
    }

    pub unsafe fn rs_set_scissor_rects(&self, scissor_rects: SScissorRects) {
        self.commandlist.RSSetScissorRects(
            scissor_rects.d3drects.len() as u32,
            &scissor_rects.d3drects[0]
        )
    }

    pub unsafe fn close(&self) -> Result<(), &'static str> {
        let hn = self.commandlist.Close();
        returnerrifwinerror!(hn, "Could not close command list.");
        Ok(())
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12GraphicsCommandList> {
        &self.commandlist
    }

    pub unsafe fn rawmut(&mut self) -> &mut ComPtr<ID3D12GraphicsCommandList> {
        &mut self.commandlist
    }
}

#[derive(Clone)]
pub struct SFence {
    fence: ComPtr<ID3D12Fence>,
}

impl SFence {
    pub fn getcompletedvalue(&self) -> u64 {
        unsafe { self.fence.GetCompletedValue() }
    }

    pub fn seteventoncompletion(
        &self,
        val: u64,
        event: &safewindows::SEventHandle,
    ) -> Result<(), &'static str> {
        let hn = unsafe { self.fence.SetEventOnCompletion(val, event.raw()) };
        returnerrifwinerror!(hn, "Could not set fence event on completion");
        Ok(())
    }
}

impl SCommandQueue {
    pub fn getdesc(&self) -> SCommandQueueDesc {
        SCommandQueueDesc {
            raw: unsafe { self.queue.GetDesc() },
        }
    }

    // -- $$$FRK(TODO): revisit this after I understand how I'm going to be using this fence
    pub fn signal(&self, fence: &SFence, val: u64) -> Result<u64, &'static str> {
        let hn = unsafe { self.queue.Signal(fence.fence.as_raw(), val) };

        returnerrifwinerror!(hn, "Could not push signal.");

        Ok(val)
    }

    // -- $$$FRK(TODO): support listS
    pub unsafe fn executecommandlist(&self, list: &SCommandList) {
        self.queue
            .ExecuteCommandLists(1, &(list.commandlist.as_raw() as *mut ID3D12CommandList));
    }
}

pub struct SGPUVirtualAddress {
    raw: D3D12_GPU_VIRTUAL_ADDRESS,
}

impl SGPUVirtualAddress {
    pub fn raw(&self) -> D3D12_GPU_VIRTUAL_ADDRESS {
        self.raw
    }
}

pub struct SVertexBufferView {
    raw: D3D12_VERTEX_BUFFER_VIEW,
}

impl SVertexBufferView {
    pub fn create(
        bufferlocation: SGPUVirtualAddress,
        sizeinbytes: u32,
        strideinbytes: u32,
    ) -> Self {
        Self {
            raw: D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: bufferlocation.raw(),
                SizeInBytes: sizeinbytes,
                StrideInBytes: strideinbytes,
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EFormat {
    R16UINT,
}

impl EFormat {
    pub fn d3dtype(&self) -> dxgiformat::DXGI_FORMAT {
        match self {
            EFormat::R16UINT => dxgiformat::DXGI_FORMAT_R16_UINT,
        }
    }
}

pub struct SIndexBufferView {
    raw: D3D12_INDEX_BUFFER_VIEW,
}

impl SIndexBufferView {
    pub fn create(bufferlocation: SGPUVirtualAddress, format: EFormat, sizeinbytes: u32) -> Self {
        Self {
            raw: D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: bufferlocation.raw(),
                Format: format.d3dtype(),
                SizeInBytes: sizeinbytes,
            },
        }
    }
}

pub struct SRootSignature {
    pub raw: ComPtr<ID3D12RootSignature>,
}

pub struct SPipelineState {
    raw: ComPtr<ID3D12PipelineState>,
}

pub struct SViewport {
    viewport: D3D12_VIEWPORT,
}

impl SViewport {
    pub fn new(
        topleftx: f32,
        toplefty: f32,
        width: f32,
        height: f32,
        mindepth: Option<f32>,
        maxdepth: Option<f32>,
    ) -> Self {
        SViewport {
            viewport: D3D12_VIEWPORT {
                TopLeftX: topleftx,
                TopLeftY: toplefty,
                Width: width,
                Height: height,
                MinDepth: mindepth.unwrap_or(D3D12_MIN_DEPTH),
                MaxDepth: maxdepth.unwrap_or(D3D12_MAX_DEPTH),
            },
        }
    }
}

pub type SRect = safewindows::SRect;

impl SRect {
    pub fn d3dtype(&self) -> D3D12_RECT {
        D3D12_RECT {
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDXGIFormat {
    Unknown,
    R32G32B32A32Typeless,
    R32G32B32Float,
    D32Float,
    R8G8B8A8UNorm,
}

impl EDXGIFormat {
    pub fn d3dtype(&self) -> dxgiformat::DXGI_FORMAT {
        match self {
            Self::Unknown => dxgiformat::DXGI_FORMAT_UNKNOWN,
            Self::R32G32B32A32Typeless => dxgiformat::DXGI_FORMAT_R32G32B32A32_TYPELESS,
            Self::R32G32B32Float => dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            Self::D32Float => dxgiformat::DXGI_FORMAT_D32_FLOAT,
            Self::R8G8B8A8UNorm => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
        }
    }
}

pub struct SDepthStencilValue {
    pub depth: f32,
    pub stencil: u8,
}

impl SDepthStencilValue {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_VALUE {
        D3D12_DEPTH_STENCIL_VALUE {
            Depth: self.depth,
            Stencil: self.stencil,
        }
    }
}

pub enum EClearValue {
    Color([f32; 4]),
    DepthStencil(SDepthStencilValue),
}

pub struct SClearValue {
    pub format: EDXGIFormat,
    pub value: EClearValue,
}

impl SClearValue {
    pub fn d3dtype(&self) -> D3D12_CLEAR_VALUE {
        unsafe {
            let mut result : D3D12_CLEAR_VALUE = mem::uninitialized();
            result.Format = self.format.d3dtype();
            match &self.value {
                EClearValue::Color(color) => *(result.u.Color_mut()) = *color,
                EClearValue::DepthStencil(depth_stencil_value) => *(result.u.DepthStencil_mut()) = depth_stencil_value.d3dtype(),
            }
            result
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDSVDimension {
    Unknown,
    Texture1D,
    Texture1DArray,
    Texture2D,
    Texture2DArray,
    Texture2DMS,
    Texture2DMSArray,
}

impl EDSVDimension {
    pub fn d3dtype(&self) -> D3D12_DSV_DIMENSION {
        match self {
            Self::Unknown => D3D12_DSV_DIMENSION_UNKNOWN,
            Self::Texture1D => D3D12_DSV_DIMENSION_TEXTURE1D,
            Self::Texture1DArray => D3D12_DSV_DIMENSION_TEXTURE1DARRAY,
            Self::Texture2D => D3D12_DSV_DIMENSION_TEXTURE2D,
            Self::Texture2DArray => D3D12_DSV_DIMENSION_TEXTURE2DARRAY,
            Self::Texture2DMS => D3D12_DSV_DIMENSION_TEXTURE2DMS,
            Self::Texture2DMSArray => D3D12_DSV_DIMENSION_TEXTURE2DMSARRAY
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDSVFlags {
    None,
    ReadOnlyDepth,
    ReadOnlyStencil,
}

impl TD3DFlags32 for EDSVFlags {
    type TD3DType = D3D12_DSV_FLAGS;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            Self::None => D3D12_DSV_FLAG_NONE,
            Self::ReadOnlyDepth => D3D12_DSV_FLAG_READ_ONLY_DEPTH,
            Self::ReadOnlyStencil => D3D12_DSV_FLAG_READ_ONLY_STENCIL
        }
    }
}
pub type SDSVFlags = SD3DFlags32<EDSVFlags>;

pub struct STex2DDSV {
    pub mip_slice: u32,
}

impl STex2DDSV {
    pub fn d3dtype(&self) -> D3D12_TEX2D_DSV {
        D3D12_TEX2D_DSV {
            MipSlice: self.mip_slice,
        }
    }
}

pub enum EDepthStencilViewDescData {
    Tex2D(STex2DDSV),
}

pub struct SDepthStencilViewDesc {
    pub format: EDXGIFormat,
    pub view_dimension: EDSVDimension,
    pub flags: SDSVFlags,
    pub data: EDepthStencilViewDescData,
}

impl SDepthStencilViewDesc {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_VIEW_DESC {
        unsafe {
            let mut result : D3D12_DEPTH_STENCIL_VIEW_DESC = mem::uninitialized();
            result.Format = self.format.d3dtype();
            result.ViewDimension = self.view_dimension.d3dtype();
            result.Flags = self.flags.d3dtype();

            match &self.data {
                EDepthStencilViewDescData::Tex2D(tex2d_dsv) => *(result.u.Texture2D_mut()) = tex2d_dsv.d3dtype(),
            }

            result
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EInputClassification {
    PerVertexData,
    PerInstanceData,
}

impl EInputClassification {
    pub fn d3dtype(&self) -> D3D12_INPUT_CLASSIFICATION {
        match self {
            Self::PerVertexData => D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            Self::PerInstanceData => D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SInputElementDesc {
    semantic_name: &'static str,
    semantic_index: u32,
    format: EDXGIFormat,
    input_slot: u32,
    aligned_byte_offset: u32,
    input_slot_class: EInputClassification,
    instance_data_step_rate: u32,

    semantic_name_null_terminated: [winapi::um::winnt::CHAR; 32],
}

impl SInputElementDesc {
    pub fn create(
        semantic_name: &'static str,
        semantic_index: u32,
        format: EDXGIFormat,
        input_slot: u32,
        aligned_byte_offset: u32,
        input_slot_class: EInputClassification,
        instance_data_step_rate: u32,
    ) -> Self {

        let mut result = Self {
            semantic_name: semantic_name,
            semantic_index: semantic_index,
            format: format,
            input_slot: input_slot,
            aligned_byte_offset: aligned_byte_offset,
            input_slot_class: input_slot_class,
            instance_data_step_rate: instance_data_step_rate,

            semantic_name_null_terminated: [0; 32],
        };

        let mut i = 0;
        for c in semantic_name.as_bytes() {
            result.semantic_name_null_terminated[i] = *c as i8;
            i += 1;
        }
        result.semantic_name_null_terminated[i] = 0;

        result
    }

    pub unsafe fn d3dtype(&self) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            //SemanticName: self.semantic_name_utf16.as_ptr(),
            SemanticName: self.semantic_name_null_terminated.as_ptr(),
            SemanticIndex: self.semantic_index,
            Format: self.format.d3dtype(),
            InputSlot: self.input_slot,
            AlignedByteOffset: self.aligned_byte_offset,
            InputSlotClass: self.input_slot_class.d3dtype(),
            InstanceDataStepRate: self.instance_data_step_rate,
        }
    }
}

pub struct SSubResourceData {
    raw: D3D12_SUBRESOURCE_DATA,
}

impl SSubResourceData {
    pub unsafe fn create<T>(data: *const T, rowpitch: usize, slicepitch: usize) -> Self {
        let subresourcedata = D3D12_SUBRESOURCE_DATA {
            pData: data as *const c_void,
            RowPitch: rowpitch as isize,
            SlicePitch: slicepitch as isize,
        };
        SSubResourceData {
            raw: subresourcedata,
        }
    }

    pub unsafe fn raw_mut(&mut self) -> &mut D3D12_SUBRESOURCE_DATA {
        &mut self.raw
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ECompile {
    Debug,
    SkipValidation,
    SkipOptimization,
    PackMatrixRowMajor,
    PackMatrixColumnMajor,
    PartialPrecision,
    ForceVsSoftwareNoOpt,
    ForcePsSoftwareNoOpt,
    NoPreshader,
    AvoidFlowControl,
    PreferFlowControl,
    EnableStrictness,
    EnableBackwardsCompatibility,
    IEEEStrictness,
    OptimizationLevel0,
    OptimizationLevel1,
    OptimizationLevel2,
    OptimizationLevel3,
    WarningsAreErrors,
    ResourcesMayAlias,
    //EnableUnboundedDescriptorTables,
    AllResourcesBound,
}

impl TD3DFlags32 for ECompile {
    type TD3DType = DWORD;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            ECompile::Debug => d3dcompiler::D3DCOMPILE_DEBUG,
            ECompile::SkipValidation => d3dcompiler::D3DCOMPILE_SKIP_VALIDATION,
            ECompile::SkipOptimization => d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION,
            ECompile::PackMatrixRowMajor => d3dcompiler::D3DCOMPILE_PACK_MATRIX_ROW_MAJOR,
            ECompile::PackMatrixColumnMajor => d3dcompiler::D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
            ECompile::PartialPrecision => d3dcompiler::D3DCOMPILE_PARTIAL_PRECISION,
            ECompile::ForceVsSoftwareNoOpt => d3dcompiler::D3DCOMPILE_FORCE_VS_SOFTWARE_NO_OPT,
            ECompile::ForcePsSoftwareNoOpt => d3dcompiler::D3DCOMPILE_FORCE_PS_SOFTWARE_NO_OPT,
            ECompile::NoPreshader => d3dcompiler::D3DCOMPILE_NO_PRESHADER,
            ECompile::AvoidFlowControl => d3dcompiler::D3DCOMPILE_AVOID_FLOW_CONTROL,
            ECompile::PreferFlowControl => d3dcompiler::D3DCOMPILE_PREFER_FLOW_CONTROL,
            ECompile::EnableStrictness => d3dcompiler::D3DCOMPILE_ENABLE_STRICTNESS,
            ECompile::EnableBackwardsCompatibility => {
                d3dcompiler::D3DCOMPILE_ENABLE_BACKWARDS_COMPATIBILITY
            }
            ECompile::IEEEStrictness => d3dcompiler::D3DCOMPILE_IEEE_STRICTNESS,
            ECompile::OptimizationLevel0 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL0,
            ECompile::OptimizationLevel1 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL1,
            ECompile::OptimizationLevel2 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL2,
            ECompile::OptimizationLevel3 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL3,
            ECompile::WarningsAreErrors => d3dcompiler::D3DCOMPILE_WARNINGS_ARE_ERRORS,
            ECompile::ResourcesMayAlias => d3dcompiler::D3DCOMPILE_RESOURCES_MAY_ALIAS,
            //ECompile::EnableUnboundedDescriptorTables => d3dcompiler::D3DCOMPILE_ENABLE_UNBOUND_DESCRIPTOR_TABLES,
            ECompile::AllResourcesBound => d3dcompiler::D3DCOMPILE_ALL_RESOURCES_BOUND,
        }
    }
}

pub type SCompile = SD3DFlags32<ECompile>;

#[derive(Copy, Clone, PartialEq)]
pub enum ERootSignatureFlags {
    ENone,
    AllowInputAssemblerInputLayout,
    DenyVertexShaderRootAccess,
    DenyHullShaderRootAccess,
    DenyDomainShaderRootAccess,
    DenyGeometryShaderRootAccess,
    DenyPixelShaderRootAccess,
    AllowStreamOutput,
    //LocalRootSignature,
}

impl TD3DFlags32 for ERootSignatureFlags {
    type TD3DType = u32;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            Self::ENone => D3D12_ROOT_SIGNATURE_FLAG_NONE,
            Self::AllowInputAssemblerInputLayout => D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            Self::DenyVertexShaderRootAccess => D3D12_ROOT_SIGNATURE_FLAG_DENY_VERTEX_SHADER_ROOT_ACCESS,
            Self::DenyHullShaderRootAccess => D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS,
            Self::DenyDomainShaderRootAccess => D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS,
            Self::DenyGeometryShaderRootAccess => D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS,
            Self::DenyPixelShaderRootAccess => D3D12_ROOT_SIGNATURE_FLAG_DENY_PIXEL_SHADER_ROOT_ACCESS,
            Self::AllowStreamOutput => D3D12_ROOT_SIGNATURE_FLAG_ALLOW_STREAM_OUTPUT,
            //Self::LocalRootSignature => D3D12_ROOT_SIGNATURE_FLAG_LOCAL_ROOT_SIGNATURE
        }
    }
}

pub type SRootSignatureFlags = SD3DFlags32<ERootSignatureFlags>;

pub struct SBlob {
    raw: ComPtr<d3dcommon::ID3DBlob>,
}

pub enum EDescriptorRangeType {
    SRV,
    UAV,
    CBV,
    Sampler,
}

impl EDescriptorRangeType {
    pub fn d3dtype(&self) -> D3D12_DESCRIPTOR_RANGE_TYPE {
        match self {
            Self::SRV => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            Self::UAV => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            Self::CBV => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
            Self::Sampler => D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,
        }
    }
}

pub enum EDescriptorRangeOffset {
    EAppend,
    ENumDecriptors{ num: u32 },
}

impl EDescriptorRangeOffset {
    pub fn d3dtype(&self) -> u32 {
        match self {
            Self::EAppend => D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            Self::ENumDecriptors{num} => *num,
        }
    }
}

pub struct SDescriptorRange {
    range_type: EDescriptorRangeType,
    num_descriptors: u32,
    base_shader_register: u32,
    register_space: u32,
    offset_in_descriptors_from_table_start: EDescriptorRangeOffset,
}

impl SDescriptorRange {
    pub fn d3dtype(&self) -> D3D12_DESCRIPTOR_RANGE {
        D3D12_DESCRIPTOR_RANGE {
            RangeType: self.range_type.d3dtype(),
            NumDescriptors: self.num_descriptors,
            BaseShaderRegister: self.base_shader_register,
            RegisterSpace: self.register_space,
            OffsetInDescriptorsFromTableStart: self.offset_in_descriptors_from_table_start.d3dtype(),
        }
    }
}

pub struct SRootConstants {
    pub shader_register: u32,
    pub register_space: u32,
    pub num_32_bit_values: u32,
}

impl SRootConstants {
    pub fn d3dtype(&self) -> D3D12_ROOT_CONSTANTS {
        D3D12_ROOT_CONSTANTS {
            ShaderRegister: self.shader_register,
            RegisterSpace: self.register_space,
            Num32BitValues: self.num_32_bit_values,
        }
    }
}

pub enum EShaderVisibility {
    All,
    Vertex,
    Hull,
    Domain,
    Geometry,
    Pixel,
}

impl EShaderVisibility {
    pub fn d3dtype(&self) -> D3D12_SHADER_VISIBILITY {
        match self {
            Self::All => D3D12_SHADER_VISIBILITY_ALL,
            Self::Vertex => D3D12_SHADER_VISIBILITY_VERTEX,
            Self::Hull => D3D12_SHADER_VISIBILITY_HULL,
            Self::Domain => D3D12_SHADER_VISIBILITY_DOMAIN,
            Self::Geometry => D3D12_SHADER_VISIBILITY_GEOMETRY,
            Self::Pixel => D3D12_SHADER_VISIBILITY_PIXEL
        }
    }
}

pub enum ERootParameterType {
    DescriptorTable,
    E32BitConstants,
    CBV,
    SRV,
    UAV,
}

impl ERootParameterType {
    pub fn d3dtype(&self) -> D3D12_ROOT_PARAMETER_TYPE {
        match self {
            Self::DescriptorTable => D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Self::E32BitConstants => D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Self::CBV => D3D12_ROOT_PARAMETER_TYPE_CBV,
            Self::SRV => D3D12_ROOT_PARAMETER_TYPE_SRV,
            Self::UAV => D3D12_ROOT_PARAMETER_TYPE_UAV
        }
    }
}

pub enum ERootParameterTypeData {
    Constants{constants: SRootConstants},
}

pub struct SRootParameter {
    pub type_: ERootParameterType,
    pub type_data: ERootParameterTypeData,
    pub shader_visibility: EShaderVisibility,
}

impl SRootParameter {
    pub fn d3dtype(&self) -> D3D12_ROOT_PARAMETER {
        unsafe {
            let mut result : D3D12_ROOT_PARAMETER = mem::uninitialized();
            result.ParameterType = self.type_.d3dtype();
            match &self.type_data {
                ERootParameterTypeData::Constants{constants} => {
                    *result.u.Constants_mut() = constants.d3dtype();
                }
            }
            result.ShaderVisibility = self.shader_visibility.d3dtype();

            result
        }
    }
}

pub struct SRootSignatureDesc {
    pub parameters: Vec<SRootParameter>,
    //static_samplers: Vec<SStaticSamplerDesc>,
    pub flags: SRootSignatureFlags,

    // -- for d3dtype()
    d3d_parameters: Vec<D3D12_ROOT_PARAMETER>,
}

impl SRootSignatureDesc {
    pub fn new(flags: SRootSignatureFlags) -> Self {
        Self {
            parameters: Vec::new(), // $$$FRK(TODO): allocations
            flags: flags,
            d3d_parameters: Vec::new(),
        }
    }

    pub unsafe fn d3dtype(&mut self) -> D3D12_ROOT_SIGNATURE_DESC {
        self.d3d_parameters.clear();
        for parameter in &self.parameters {
            self.d3d_parameters.push(parameter.d3dtype());
        }

        D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: self.parameters.len() as u32,
            pParameters: self.d3d_parameters.as_ptr(),
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null_mut(),
            Flags: self.flags.d3dtype(),
        }
    }
}

pub enum ERootSignatureVersion {
    V1,
    V1_0,
    V1_1,
}

impl ERootSignatureVersion {
    pub fn d3dtype(&self) -> D3D_ROOT_SIGNATURE_VERSION {
        match self {
            Self:: V1 => D3D_ROOT_SIGNATURE_VERSION_1,
            Self:: V1_0 => D3D_ROOT_SIGNATURE_VERSION_1_0,
            Self:: V1_1 => D3D_ROOT_SIGNATURE_VERSION_1_1,
        }
    }
}

pub fn serialize_root_signature(
    root_signature: &mut SRootSignatureDesc,
    version: ERootSignatureVersion) -> Result<SBlob, SBlob> {

    let mut raw_result_blob: *mut d3dcommon::ID3DBlob = ptr::null_mut();
    let mut raw_err_blob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let d3d_signature = unsafe { root_signature.d3dtype() };

    let hr = unsafe {
        D3D12SerializeRootSignature(
            &d3d_signature,
            version.d3dtype(),
            &mut raw_result_blob,
            &mut raw_err_blob,
        )
    };

    if winerror::SUCCEEDED(hr) {
        Ok(SBlob {
            raw: unsafe { ComPtr::from_raw(raw_result_blob) },
        })
    }
    else {
        Err(SBlob {
            raw: unsafe { ComPtr::from_raw(raw_err_blob) },
        })
    }
}

pub struct SShaderBytecode<'a> {
    bytecode: &'a SBlob,
}

impl<'a> SShaderBytecode<'a> {
    pub fn create(blob: &'a SBlob) -> Self {
        Self {
            bytecode: blob,
        }
    }

    pub unsafe fn d3dtype(&self) -> D3D12_SHADER_BYTECODE {
        let ptr = self.bytecode.raw.GetBufferPointer();
        let len = self.bytecode.raw.GetBufferSize();

        D3D12_SHADER_BYTECODE {
            pShaderBytecode: ptr,
            BytecodeLength: len,
        }
    }
}

pub struct SInputLayoutDesc {
    input_element_descs: ArrayVec::<[SInputElementDesc; 16]>,

    d3d_input_element_descs: ArrayVec::<[D3D12_INPUT_ELEMENT_DESC; 16]>,
}

impl SInputLayoutDesc {
    // -- $$$FRK(TODO): This probably belongs in niced3d12
    pub fn create(input_element_descs: &[SInputElementDesc]) -> Self {
        let mut result = Self {
            input_element_descs: ArrayVec::new(),
            d3d_input_element_descs: ArrayVec::new(),
        };

        result.input_element_descs.try_extend_from_slice(input_element_descs).unwrap();
        result
    }

    pub unsafe fn generate_d3dtype(&mut self) {
        self.d3d_input_element_descs.clear();

        for input_element_desc in &self.input_element_descs {
            self.d3d_input_element_descs.push(input_element_desc.d3dtype());
        }
    }

    pub unsafe fn d3dtype(&mut self) -> D3D12_INPUT_LAYOUT_DESC {
        // -- $$$FRK(NOTE): the generate data here is no longer valid if this moves!!!
        // -- it contains internal references!
        self.generate_d3dtype();

        let result = D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: self.d3d_input_element_descs.as_ptr(),
            NumElements: self.d3d_input_element_descs.len() as u32,
        };

        result
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EPrimitiveTopologyType {
    Undefined,
    Point,
    Line,
    Triangle,
    Patch,
}

impl EPrimitiveTopologyType {
    pub fn d3dtype(&self) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
        match self {
            Self::Undefined => D3D12_PRIMITIVE_TOPOLOGY_TYPE_UNDEFINED,
            Self::Point => D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
            Self::Line => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            Self::Triangle => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            Self::Patch => D3D12_PRIMITIVE_TOPOLOGY_TYPE_PATCH
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EPrimitiveTopology {
    // -- not comprehensive, too many to type at once, add as needed
    TriangleList,
}

impl EPrimitiveTopology {
    pub fn d3dtype(&self) -> D3D12_PRIMITIVE_TOPOLOGY {
        match self {
            Self::TriangleList => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDepthWriteMask {
    Zero,
    All,
}

impl EDepthWriteMask {
    pub fn d3dtype(&self) -> D3D12_DEPTH_WRITE_MASK {
        match self {
            Self::Zero => D3D12_DEPTH_WRITE_MASK_ZERO,
            Self::All => D3D12_DEPTH_WRITE_MASK_ALL,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EComparisonFunc {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl EComparisonFunc {
    pub fn d3dtype(&self) -> D3D12_COMPARISON_FUNC {
        match self {
            Self::Never => D3D12_COMPARISON_FUNC_NEVER,
            Self::Less => D3D12_COMPARISON_FUNC_LESS,
            Self::Equal => D3D12_COMPARISON_FUNC_EQUAL,
            Self::LessEqual => D3D12_COMPARISON_FUNC_LESS_EQUAL,
            Self::Greater => D3D12_COMPARISON_FUNC_GREATER,
            Self::NotEqual => D3D12_COMPARISON_FUNC_NOT_EQUAL,
            Self::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
            Self::Always => D3D12_COMPARISON_FUNC_ALWAYS,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum EStencilOp {
    Keep,
    Zero,
    Replace,
    IncrSat,
    DecrSat,
    Invert,
    Incr,
    Decr,
}

impl EStencilOp {
    pub fn d3dtype(&self) -> D3D12_STENCIL_OP {
        match self {
            Self::Keep => D3D12_STENCIL_OP_KEEP,
            Self::Zero => D3D12_STENCIL_OP_ZERO,
            Self::Replace => D3D12_STENCIL_OP_REPLACE,
            Self::IncrSat => D3D12_STENCIL_OP_INCR_SAT,
            Self::DecrSat => D3D12_STENCIL_OP_DECR_SAT,
            Self::Invert => D3D12_STENCIL_OP_INVERT,
            Self::Incr => D3D12_STENCIL_OP_INCR,
            Self::Decr => D3D12_STENCIL_OP_DECR,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct SDepthStencilOpDesc {
    stencil_fail_op: EStencilOp,
    stencil_depth_fail_op: EStencilOp,
    stencil_pass_op: EStencilOp,
    stencil_func: EComparisonFunc,
}

impl SDepthStencilOpDesc {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCILOP_DESC {
        D3D12_DEPTH_STENCILOP_DESC {
            StencilFailOp: self.stencil_fail_op.d3dtype(),
            StencilDepthFailOp: self.stencil_depth_fail_op.d3dtype(),
            StencilPassOp: self.stencil_pass_op.d3dtype(),
            StencilFunc: self.stencil_func.d3dtype(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SDepthStencilDesc {
    depth_enable: bool,
    depth_write_mask: EDepthWriteMask,
    depth_func: EComparisonFunc,
    stencil_enable: bool,
    stencil_read_mask: u8,
    stencil_write_mask: u8,
    front_face: SDepthStencilOpDesc,
    back_face: SDepthStencilOpDesc,
}

impl SDepthStencilDesc {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: self.depth_enable as BOOL,
            DepthWriteMask: self.depth_write_mask.d3dtype(),
            DepthFunc: self.depth_func.d3dtype(),
            StencilEnable: self.stencil_enable as BOOL,
            StencilReadMask: self.stencil_read_mask,
            StencilWriteMask: self.stencil_write_mask,
            FrontFace: self.front_face.d3dtype(),
            BackFace: self.back_face.d3dtype(),
        }
    }
}

pub struct SRTFormatArray {
    pub rt_formats: ArrayVec<[EDXGIFormat; 8]>,
}

impl SRTFormatArray {
    pub fn d3dtype(&self) -> D3D12_RT_FORMAT_ARRAY {
        let mut result : D3D12_RT_FORMAT_ARRAY = unsafe { mem::uninitialized() };
        result.NumRenderTargets = self.rt_formats.len() as UINT;

        for i in 0..self.rt_formats.len() {
            result.RTFormats[i] = self.rt_formats[i].d3dtype();
        }
        for i in self.rt_formats.len()..8 {
            result.RTFormats[i] = EDXGIFormat::Unknown.d3dtype();
        }

        result
    }
}

pub struct SPipelineStateStreamDesc<'a, T> {
    stream: &'a T,
}

impl<'a, T> SPipelineStateStreamDesc<'a, T> {
    pub fn create(stream: &'a T) -> Self {
        Self {
            stream: stream,
        }
    }

    pub unsafe fn d3dtype(&self) -> D3D12_PIPELINE_STATE_STREAM_DESC {
        let mut desc : D3D12_PIPELINE_STATE_STREAM_DESC = mem::uninitialized();
        desc.SizeInBytes = mem::size_of::<T>() as winapi::shared::basetsd::SIZE_T;
        desc.pPipelineStateSubobjectStream = self.stream as *const T as *mut c_void;

        desc
    }
}

/*
pub struct SGraphicsPipelineStateDesc {
    root_signature: &SRootSignature,
    v_s: Option<&SShaderBytecode>,
    p_s: Option<&SShaderBytecode>,
    d_s: Option<&SShaderBytecode>,
    h_s: Option<&SShaderBytecode>,
    g_s: Option<&SShaderBytecode>,
    stream_output: Option<SStreamOutputDesc>,
    blend_state: Option<SBlendDesc>,
}
*/

pub enum EPipelineStateSubobjectType {
    RootSignature,
    VS,
    PS,
    DS,
    HS,
    GS,
    CS,
    StreamOutput,
    Blend,
    SampleMask,
    Rasterizer,
    DepthStencil,
    InputLayout,
    IBStripCutValue,
    PrimitiveTopology,
    RenderTargetFormats,
    DepthStencilFormat,
    SampleDesc,
    NodeMask,
    CachedPSO,
    Flags,
    DepthStencil1,
    //ViewInstancing,
    MaxValid,
}

impl EPipelineStateSubobjectType {
    pub fn d3dtype(&self) -> D3D12_PIPELINE_STATE_SUBOBJECT_TYPE {
        match self {
            Self::RootSignature => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE,
            Self::VS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS,
            Self::PS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS,
            Self::DS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DS,
            Self::HS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_HS,
            Self::GS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_GS,
            Self::CS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CS,
            Self::StreamOutput => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_STREAM_OUTPUT,
            Self::Blend => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND,
            Self::SampleMask => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK,
            Self::Rasterizer => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER,
            Self::DepthStencil => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL,
            Self::InputLayout => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT,
            Self::IBStripCutValue => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE,
            Self::PrimitiveTopology => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY,
            Self::RenderTargetFormats => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS,
            Self::DepthStencilFormat => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT,
            Self::SampleDesc => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC,
            Self::NodeMask => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_NODE_MASK,
            Self::CachedPSO => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CACHED_PSO,
            Self::Flags => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_FLAGS,
            Self::DepthStencil1 => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL1,
            //Self::ViewInstancing => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VIEW_INSTANCING,
            Self::MaxValid => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MAX_VALID,
        }
    }
}

// -- $$$FRK(TODO): unsupported:
// --    + pDefines
// --    + pInclude
// --    + flags2
pub fn d3dcompilefromfile(
    file: &str,
    entrypoint: &str,
    target: &str,
    flags1: SCompile,
) -> Result<SBlob, &'static str> {
    // -- $$$FRK(TODO): allocations :(
    let mut fileparam: Vec<u16> = file.encode_utf16().collect();
    fileparam.push('\0' as u16);

    let mut entrypointparam: Vec<char> = entrypoint.chars().collect();
    entrypointparam.push('\0');

    let mut targetparam: Vec<char> = target.chars().collect();
    targetparam.push('\0');

    let mut rawcodeblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();
    let mut errormsgsblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let hr = unsafe {
        d3dcompiler::D3DCompileFromFile(
            fileparam.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            entrypointparam.as_ptr() as *const i8,
            targetparam.as_ptr() as *const i8,
            flags1.d3dtype(),
            0,
            &mut rawcodeblob,
            &mut errormsgsblob,
        )
    };

    returnerrifwinerror!(hr, "failed to compile shader");
    // -- $$$FRK(TODO): use error messages blob

    Ok(SBlob {
        raw: unsafe { ComPtr::from_raw(rawcodeblob) },
    })
}

pub fn read_file_to_blob(
    file: &str,
) -> Result<SBlob, &'static str> {
    let mut fileparam: Vec<u16> = file.encode_utf16().collect();
    fileparam.push('\0' as u16);

    let mut resultblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let hr = unsafe {
        d3dcompiler::D3DReadFileToBlob(
            fileparam.as_ptr(),
            &mut resultblob,
        )
    };

    returnerrifwinerror!(hr, "failed to load shader");

    Ok(SBlob {
        raw: unsafe { ComPtr::from_raw(resultblob) },
    })
}
