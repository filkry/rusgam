#![allow(dead_code)]

use safewindows;

use std::{mem, ptr};

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
use winapi::um::{d3dcommon, unknwnbase, d3dcompiler};
use winapi::Interface;

use wio::com::ComPtr;

use std::marker::PhantomData;

// -- this is copied in safewindows, does it have to be?
trait ComPtrPtrs<T> {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T>
where
    T: Interface,
{
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown {
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

    pub fn d3d12createdevice(&mut self) -> Result<SDevice, &'static str> {
        unsafe { d3d12createdevice(self.adapter.asunknownptr()) }
    }
}

pub struct SAdapter4 {
    adapter: ComPtr<IDXGIAdapter4>,
}

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
    pub fn d3d12createdevice(&mut self) -> Result<SDevice, &'static str> {
        unsafe { d3d12createdevice(self.adapter.asunknownptr()) }
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

pub struct SCommandQueue {
    queue: ComPtr<ID3D12CommandQueue>,
}

impl SDevice {
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
}

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
            SGPUVirtualAddress{
                raw: self.resource.GetGPUVirtualAddress(),
            }
        }
    }
}

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
    pub fn createswapchainforwindow(
        &self,
        window: &safewindows::SWindow,
        commandqueue: &mut SCommandQueue,
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

        let hr = unsafe {
            self.factory.CreateSwapChainForHwnd(
                commandqueue.queue.asunknownptr(),
                window.raw(),
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut rawswapchain as *mut *mut _ as *mut *mut IDXGISwapChain1,
            )
        };

        returnerrifwinerror!(hr, "Failed to create swap chain");

        let swapchain = unsafe { ComPtr::from_raw(rawswapchain) };

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

pub struct SDescriptorHeap {
    pub type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
}

impl SDescriptorHeap {
    pub fn getcpudescriptorhandleforheapstart(&self) -> SDescriptorHandle {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SDescriptorHandle {
            handle: start,
        }
    }
}

pub struct SDescriptorHandle<'heap> {
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl SDescriptorHandle {
    pub unsafe fn offset(&self, bytes: usize) -> SDescriptorHandle {
        SDescriptorHandle {
            handle: D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + bytes,
            },
            phantom: PhantomData,
        }
    }
}

// -- $$$FRK(TODO): combine impls
impl SDevice {
    pub fn createdescriptorheap(
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

    // -- $$$FRK(TODO): Wrapper for D3D12 Resource Flags?
    pub fn createcommittedresource(
        &self,
        heapproperties: SHeapProperties,
        heapflags: EHeapFlags,
        resourcedesc: SResourceDesc,
        initialresourcestate: EResourceStates,
        _optimizedclearvalue: Option<u32>, // -- $$$FRK(TODO): clear value
    ) -> Result<SResource, &'static str>
    {

        unsafe {
            let mut rawresource: *mut ID3D12Resource = ptr::null_mut();
            let hn = self.device.CreateCommittedResource(
                &heapproperties.raw,
                heapflags.d3dtype(),
                &resourcedesc.raw,
                initialresourcestate.d3dtype(),
                ptr::null() as *const D3D12_CLEAR_VALUE,
                &ID3D12Resource::uuidof(), // $$$FRK(TODO): this isn't necessarily right
                &mut rawresource as *mut *mut _ as *mut *mut c_void,
            );

            returnerrifwinerror!(hn, "Could not create committed resource.");
            Ok(SResource {
                resource: ComPtr::from_raw(rawresource),
            })
        }
    }
}

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
        Self{
            raw: D3D12_HEAP_PROPERTIES{
                Type: type_.d3dtype(),
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            }
        }
    }
}

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
    type TD3DType : std::convert::Into<u32> + std::convert::From<u32> + Copy + Clone;

    fn d3dtype(&self) -> Self::TD3DType;
}

pub struct SD3DFlags32<T: TD3DFlags32> {
    raw: T::TD3DType,
}

impl<T: TD3DFlags32> From<T> for SD3DFlags32<T> {
    fn from(flag: T) -> Self {
        Self::none().and(flag)
    }
}

impl<T: TD3DFlags32> Clone for SD3DFlags32<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TD3DFlags32> Copy for SD3DFlags32<T> {}

impl<T: TD3DFlags32> SD3DFlags32<T> {
    pub fn none() -> Self {
        Self{
            raw: T::TD3DType::from(0),
        }
    }

    pub fn all() -> Self {
        Self{
            raw: T::TD3DType::from(std::u32::MAX),
        }
    }

    pub fn and(&self, other: T) -> Self {
        let a : u32 = self.raw.into();
        let b : u32 = other.d3dtype().into();
        let res : u32 = a & b;
        Self{
            raw: T::TD3DType::from(res),
        }
    }

    pub fn or(&self, other: T) -> Self {
        let a : u32 = self.raw.into();
        let b : u32 = other.d3dtype().into();
        let res : u32 = a | b;
        Self{
            raw: T::TD3DType::from(res),
        }
    }

    pub fn d3dtype(&self) -> T::TD3DType {
        self.raw
    }
}

impl SResourceDesc {
    pub fn createbuffer(buffersize: usize, flags: SResourceFlags) -> Self {
        Self{
            raw: D3D12_RESOURCE_DESC{
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: buffersize as u64, // seems like this is used as the main dimension for a 1D resource
                Height: 1, // required
                DepthOrArraySize: 1, // required
                MipLevels: 1, // required
                Format: dxgiformat::DXGI_FORMAT_UNKNOWN, // required
                SampleDesc: dxgitype::DXGI_SAMPLE_DESC{
                    Count: 1, // required
                    Quality: 0, // required
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR, // required
                Flags: flags.d3dtype(),
            }
        }
    }
}

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
            EResourceFlags::AllowSimultaneousAccess => D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS,
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

#[derive(Clone)]
pub struct SCommandList {
    commandlist: ComPtr<ID3D12GraphicsCommandList>,
}

impl SCommandList {
    pub fn gettype(&self) -> ECommandListType {
        unsafe {
            ECommandListType::create(self.commandlist.GetType())
        }
    }

    pub fn reset(&self, commandallocator: &SCommandAllocator) -> Result<(), &'static str> {
        let hn = unsafe {
            self.commandlist
                .Reset(commandallocator.commandallocator.as_raw(), ptr::null_mut())
        };
        returnerrifwinerror!(hn, "Could not reset command list.");
        Ok(())
    }

    pub fn resourcebarrier(&self, numbarriers: u32, barriers: &[SBarrier]) {
        // -- $$$FRK(TODO): need to figure out how to make a c array from the rust slice
        // -- w/o a heap allocation...
        assert!(numbarriers == 1);
        unsafe { self.commandlist.ResourceBarrier(1, &(barriers[0].barrier)) };
    }

    pub fn clearrendertargetview(&self, descriptor: SDescriptorHandle, colour: &[f32; 4]) {
        // -- $$$FRK(TODO): support third/fourth parameter
        unsafe {
            self.commandlist
                .ClearRenderTargetView(descriptor.handle, colour, 0, ptr::null());
        }
    }

    pub fn close(&self) -> Result<(), &'static str> {
        let hn = unsafe { self.commandlist.Close() };
        returnerrifwinerror!(hn, "Could not close command list.");
        Ok(())
    }

    pub unsafe fn rawmut(&mut self) -> &mut ComPtr<ID3D12GraphicsCommandList> {
        &mut self.commandlist
    }
}

impl SDevice {
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

    pub fn createcommandlist<'allocator>(
        &self,
        allocator: &'allocator SCommandAllocator,
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
}

pub struct SFence {
    fence: ComPtr<ID3D12Fence>,
}

impl SDevice {
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
    pub fn executecommandlist(&self, list: &mut SCommandList) {
        unsafe {
            self.queue
                .ExecuteCommandLists(1, &(list.commandlist.as_raw() as *mut ID3D12CommandList));
        }
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
    pub fn create(bufferlocation: SGPUVirtualAddress, sizeinbytes: u32, strideinbytes: u32) -> Self {
        Self {
            raw: D3D12_VERTEX_BUFFER_VIEW{
                BufferLocation: bufferlocation.raw(),
                SizeInBytes: sizeinbytes,
                StrideInBytes: strideinbytes,
            },
        }
    }
}

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
            raw: D3D12_INDEX_BUFFER_VIEW{
                BufferLocation: bufferlocation.raw(),
                Format: format.d3dtype(),
                SizeInBytes: sizeinbytes,
            },
        }
    }
}


pub struct SRootSignature {
    rootsignature: ComPtr<ID3D12RootSignature>,
}

pub struct SPipelineState {
    pipelinestate: ComPtr<ID3D12PipelineState>,
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

pub struct SSubResourceData {
    raw: D3D12_SUBRESOURCE_DATA,
}

impl SSubResourceData {
    pub unsafe fn create<T>(data: *const T, rowpitch: usize, slicepitch: usize) -> Self {
        let subresourcedata = D3D12_SUBRESOURCE_DATA{
            pData: data as *const c_void,
            RowPitch: rowpitch as isize,
            SlicePitch: slicepitch as isize,
        };
        SSubResourceData{
            raw: subresourcedata,
        }
    }

    pub unsafe fn raw_mut(&mut self) -> &mut D3D12_SUBRESOURCE_DATA {
        &mut self.raw
    }
}

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
            ECompile::EnableBackwardsCompatibility => d3dcompiler::D3DCOMPILE_ENABLE_BACKWARDS_COMPATIBILITY,
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

pub struct SBlob {
    raw: ComPtr<d3dcommon::ID3DBlob>,
}

// -- $$$FRK(TODO): unsupported:
// --    + pDefines
// --    + pInclude
// --    + flags2
pub fn d3dcompilefromfile(file: &str, entrypoint: &str, target: &str, flags1: SCompile) -> Result<SBlob, &'static str> {
    // -- $$$FRK(TODO): allocations :(
    let mut fileparam: Vec<u16> = file.encode_utf16().collect();
    fileparam.push('\0' as u16);

    let mut entrypointparam: Vec<char> = entrypoint.chars().collect();
    entrypointparam.push('\0');

    let mut targetparam: Vec<char> = target.chars().collect();
    targetparam.push('\0');

    let mut rawcodeblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();
    let mut errormsgsblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let hr = unsafe { d3dcompiler::D3DCompileFromFile(
        fileparam.as_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
        entrypointparam.as_ptr() as *const i8,
        targetparam.as_ptr() as *const i8,
        flags1.d3dtype(),
        0,
        &mut rawcodeblob,
        &mut errormsgsblob,
    ) };

    returnerrifwinerror!(hr, "failed to compile shader");
    // -- $$$FRK(TODO): use error messages blob

    Ok(SBlob{
        raw: unsafe { ComPtr::from_raw(rawcodeblob) },
    })
}