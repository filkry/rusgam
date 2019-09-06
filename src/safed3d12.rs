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
use winapi::um::{d3dcommon, unknwnbase};
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
    fn d3dstate(&self) -> D3D12_RESOURCE_STATES {
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
        StateBefore: beforestate.d3dstate(),
        StateAfter: afterstate.d3dstate(),
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

#[derive(Copy, Clone)]
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
}

pub struct SCommandQueue<'device> {
    queue: ComPtr<ID3D12CommandQueue>,
    phantom: PhantomData<&'device SDevice>,
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
            phantom: PhantomData,
        })
    }
}

pub struct SResource {
    resource: ComPtr<ID3D12Resource>,
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

    pub fn currentbackbufferindex(&self) -> u32 {
        unsafe { self.swapchain.GetCurrentBackBufferIndex() }
    }

    pub fn getbuffer(&self, idx: u32) -> Result<SResource, &'static str> {
        let mut rawbuf: *mut ID3D12Resource = ptr::null_mut();
        let hn = unsafe {
            self.swapchain.GetBuffer(
                idx,
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

pub struct SDescriptorHeap<'device> {
    pub type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
    phantom: PhantomData<&'device SDevice>,
}

impl<'device> SDescriptorHeap<'device> {
    pub fn getcpudescriptorhandleforheapstart(&self) -> SDescriptorHandle<'_, 'device> {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SDescriptorHandle {
            handle: start,
            phantom: PhantomData,
        }
    }
}

pub struct SDescriptorHandle<'heap, 'device> {
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
    phantom: PhantomData<&'heap SDescriptorHeap<'device>>,
}

impl<'heap, 'device> SDescriptorHandle<'heap, 'device> {
    pub unsafe fn offset(&self, bytes: usize) -> SDescriptorHandle<'heap, 'device> {
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
            phantom: PhantomData,
        })
    }

    pub fn getdescriptorhandleincrementsize(&self, type_: EDescriptorHeapType) -> u32 {
        unsafe {
            self.device
                .GetDescriptorHandleIncrementSize(type_.d3dtype())
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
}

#[derive(Clone)]
pub struct SCommandAllocator<'device> {
    type_: ECommandListType,
    commandallocator: ComPtr<ID3D12CommandAllocator>,
    phantom: PhantomData<&'device SDevice>,
}

impl<'device> SCommandAllocator<'device> {
    pub fn reset(&self) {
        unsafe { self.commandallocator.Reset() };
    }
}

#[derive(Clone)]
pub struct SCommandList {
    commandlist: ComPtr<ID3D12GraphicsCommandList>,
}

impl SCommandList {
    pub fn reset(
        &self,
        commandallocator: &SCommandAllocator,
    ) -> Result<(), &'static str> {
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
            phantom: PhantomData,
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

pub struct SFence<'device> {
    fence: ComPtr<ID3D12Fence>,
    phantom: PhantomData<&'device SDevice>,
}

impl SDevice {
    // -- $$$FRK(TODO): think about mutable refs for lots of fns here and in safewindows
    pub fn createfence(&self) -> Result<SFence<'_>, &'static str> {
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
            phantom: PhantomData,
        })
    }
}

impl<'device> SFence<'device> {
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

impl<'device> SCommandQueue<'device> {
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
