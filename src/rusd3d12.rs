#![allow(dead_code)]

// -- $$$FRK(TODO): large portions of this are ported from the jpbanoosten D3D12 tutorial, which is
// under the MIT license. I need to figure out what my obligations are re:that if I ever release
// this.

//use winapi::um::d3d12 as dx;
use std::{cmp, fmt, mem, ptr};
//use std::ptr::{null};

// -- $$$FRK(TODO): I feel very slightly guilty about all these wildcard uses
use winapi::{Interface};
use winapi::ctypes::{c_void};
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::dxgi1_4::*;
use winapi::shared::dxgi1_5::*;
use winapi::shared::dxgi1_6::*;
use winapi::shared::{dxgiformat, dxgitype, ntdef, winerror};
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::d3d12::*;
use winapi::um::d3d12sdklayers::*;
use winapi::um::{d3dcommon, errhandlingapi, libloaderapi, profileapi, synchapi, winnt, unknwnbase};
use winapi::um::winnt::LONG;
use winapi::um::winuser::*;

use wio::com::ComPtr;
//use winapi::shared::{guiddef};

macro_rules! returnerrifwinerror {
    ($hn:expr, $err:expr) => (
        if !winerror::SUCCEEDED($hn) {
            return Err($err);
        }
    );
}

trait ComPtrPtrs<T> {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown;
}

impl<T> ComPtrPtrs<T> for ComPtr<T> where T: Interface {
    unsafe fn asunknownptr(&mut self) -> *mut unknwnbase::IUnknown {
        self.as_raw() as *mut unknwnbase::IUnknown
    }
}

// -- $$$FRK(TODO): need to decide what I'm doing with errors re: HRESULT and DWORD errcodes -
// maybe a union?
pub struct SErr {
    errcode: DWORD,
}

pub unsafe fn getlasterror() -> SErr {
    SErr{errcode: errhandlingapi::GetLastError()}
}


impl fmt::Debug for SErr {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        // -- $$$FRK(TODO): we can call GetLastError to impl Debug/Display for SErr
        Ok(())
    }
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
        }
        else {
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

pub struct SWinAPI {
    hinstance: HINSTANCE,
}

pub fn initwinapi() -> Result<SWinAPI, SErr> {
    unsafe {
        let hinstance = libloaderapi::GetModuleHandleW(ntdef::NULL as *const u16);
        if !hinstance.is_null() {
            Ok(SWinAPI{hinstance: hinstance})
        }
        else {
            Err(getlasterror())
        }
    }
}

pub struct SWindowClass<'windows> {
    winapi: &'windows SWinAPI,
    windowclassname: &'static str,
    class: ATOM,
}

impl SWinAPI {
    pub fn curtimemicroseconds(&self) -> i64 {
        let mut result: winnt::LARGE_INTEGER = unsafe { mem::uninitialized() };
        let success = unsafe { profileapi::QueryPerformanceCounter(&mut result) };
        if success == 0 {
            panic!("Can't query performance for timing.");
        }

        unsafe { *result.QuadPart()}
    }

    pub fn registerclassex(&self,
                           windowclassname: &'static str) -> Result<SWindowClass, SErr> {
        unsafe {
            let classdata = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(DefWindowProcW), //wndproc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: self.hinstance,
                hIcon: LoadIconW(self.hinstance, ntdef::NULL as *const u16),
                hCursor: LoadCursorW(ntdef::NULL as HINSTANCE, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: ntdef::NULL as *const u16,
                lpszClassName: windowclassname.as_ptr() as *const winnt::WCHAR,
                hIconSm: ntdef::NULL as HICON,
            };

            let atom = RegisterClassExW(&classdata);
            if atom > 0 {
                Ok(SWindowClass{winapi: self, windowclassname: windowclassname, class: atom})
            }
            else {
                Err(getlasterror())
            }
        }
    }
}

pub struct SWindow {
    window: HWND,
}

impl<'windows> SWindowClass<'windows> {
    pub fn createwindow(&self, title: &str, width: u32, height: u32) -> Result<SWindow, SErr> {
        unsafe {
            let windowstyle: DWORD = WS_OVERLAPPEDWINDOW;

            let screenwidth = GetSystemMetrics(SM_CXSCREEN);
            let screenheight = GetSystemMetrics(SM_CYSCREEN);

            let mut windowrect = RECT{left: 0, top: 0,
                                      right: width as LONG, bottom: height as LONG};
            AdjustWindowRect(&mut windowrect, windowstyle, false as i32);

            let windowwidth = windowrect.right - windowrect.left;
            let windowheight = windowrect.bottom - windowrect.top;

            let windowx = cmp::max(0, (screenwidth - windowwidth) / 2);
            let windowy = cmp::max(0, (screenheight - windowheight) / 2);

            //self.class as ntdef::LPCWSTR,
            let windowclassnameparam = self.windowclassname.as_ptr() as ntdef::LPCWSTR;
            let titleparam = title.as_ptr() as ntdef::LPCWSTR;
            let hinstanceparam = self.winapi.hinstance;

            let hwnd: HWND = CreateWindowExW(
                0,
                windowclassnameparam,
                titleparam,
                windowstyle,
                windowx,
                windowy,
                windowwidth,
                windowheight,
                ntdef::NULL as HWND,
                ntdef::NULL as HMENU,
                hinstanceparam,
                ntdef::NULL
            );

            if !hwnd.is_null() {
                Ok(SWindow{window: hwnd})
            }
            else {
                Err(getlasterror())
            }
         }
    }
}

pub struct SD3D12 {
    factory: ComPtr<IDXGIFactory4>,
}

pub fn initd3d12() -> Result<SD3D12, &'static str> {
    let mut rawfactory: *mut IDXGIFactory4 = ptr::null_mut();
    let createfactoryresult = unsafe {
        CreateDXGIFactory2(DXGI_CREATE_FACTORY_DEBUG,
                           &IDXGIFactory4::uuidof(),
                           &mut rawfactory as *mut *mut _ as *mut *mut c_void)
    };
    if winerror::SUCCEEDED(createfactoryresult) {
        return Ok(SD3D12{factory: unsafe { ComPtr::from_raw(rawfactory) }});
    }

    Err("Couldn't get D3D12 factory.")
}

pub struct SAdapter {
    adapter: ComPtr<IDXGIAdapter4>,
}

impl SD3D12 {
    pub fn getadapter(&self) -> Result<SAdapter, &'static str> { 
        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();

            if unsafe { self.factory.EnumAdapters1(adapteridx, &mut rawadapter1) } ==
               winerror::DXGI_ERROR_NOT_FOUND {
                continue;
            }

            let mut adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };

            let mut adapterdesc: DXGI_ADAPTER_DESC1 = unsafe {mem::uninitialized() };
            unsafe { adapter1.GetDesc1(&mut adapterdesc) };

            if adapterdesc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
                continue;
            }

            let devicecreateresult = unsafe {
                D3D12CreateDevice(adapter1.asunknownptr(),
                                  d3dcommon::D3D_FEATURE_LEVEL_11_0,
                                  &ID3D12Device::uuidof(),
                                  ptr::null_mut()) };
            if !winerror::SUCCEEDED(devicecreateresult) {
                continue;
            }

            if adapterdesc.DedicatedVideoMemory > maxdedicatedmem {
                match adapter1.cast::<IDXGIAdapter4>() {
                    Ok(_) => {
                        bestadapter = adapteridx;
                        maxdedicatedmem = adapterdesc.DedicatedVideoMemory;
                    }
                    Err(_) => {}
                }
            }
        }

        if maxdedicatedmem > 0 {
            let mut rawadapter1: *mut IDXGIAdapter1 = ptr::null_mut();
            unsafe { self.factory.EnumAdapters1(bestadapter, &mut rawadapter1) };
            let adapter1: ComPtr<IDXGIAdapter1> = unsafe { ComPtr::from_raw(rawadapter1) };
            match adapter1.cast::<IDXGIAdapter4>() {
                Ok(a) => {
                    return Ok(SAdapter{adapter: a});
                }
                Err(_) => {
                    return Err("Getting Adapter4 failed despite working earlier");
                }
            };
        }

        Err("Could not find valid adapter")
    }
}

pub struct SDevice {
    device: ComPtr<ID3D12Device2>,
}

impl SAdapter {
    pub fn createdevice(&mut self) -> Result<SDevice, &'static str> {
        let mut rawdevice: *mut ID3D12Device2 = ptr::null_mut();
        let hn = unsafe {
            D3D12CreateDevice(self.adapter.asunknownptr(),
                              d3dcommon::D3D_FEATURE_LEVEL_11_0,
                              &ID3D12Device2::uuidof(),
                              &mut rawdevice as *mut *mut _ as *mut *mut c_void)
        };
        returnerrifwinerror!(hn, "Could not create device on adapter.");

        let device = unsafe { ComPtr::from_raw(rawdevice) };

        // -- $$$FRK(TODO): debug only
        match device.cast::<ID3D12InfoQueue>() {
            Ok(infoqueue) => {
                unsafe {
                    infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_CORRUPTION, TRUE);
                    infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_ERROR, TRUE);
                    infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_WARNING, TRUE);
                }

                let mut suppressedseverities = [
                    D3D12_MESSAGE_SEVERITY_INFO
                ];

                let mut suppressedmessages = [
                    D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE
                ];

                let allowlist = D3D12_INFO_QUEUE_FILTER_DESC{
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: 0,
                    pSeverityList: ptr::null_mut(),
                    NumIDs: 0,
                    pIDList: ptr::null_mut(),
                };

                let denylist = D3D12_INFO_QUEUE_FILTER_DESC{
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: suppressedseverities.len() as u32,
                    pSeverityList: &mut suppressedseverities[0] as *mut u32,
                    NumIDs: suppressedmessages.len() as u32,
                    pIDList: &mut suppressedmessages[0] as *mut u32,
                };

                let mut filter = D3D12_INFO_QUEUE_FILTER{
                    AllowList: allowlist,
                    DenyList: denylist,
                };

                let hn = unsafe {infoqueue.PushStorageFilter(&mut filter)};
                returnerrifwinerror!(hn, "Could not push storage filter on infoqueue.");
            }
            Err(_) => {
                return Err("Could not get info queue from adapter.");
            }
        }

        Ok(SDevice{device: device})
    }
}

pub enum ECommandListType {
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
            ECommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT ,
            ECommandListType::Bundle => D3D12_COMMAND_LIST_TYPE_BUNDLE ,
            ECommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE ,
            ECommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY ,
            //VideoDecode => D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE ,
            //VideoProcess => D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS ,
        }
    }
}

pub struct SCommandQueue {
    queue: ComPtr<ID3D12CommandQueue>,
}

impl SDevice {
    pub fn createcommandqueue(&self, type_: ECommandListType) -> Result<SCommandQueue, &'static str> {
        let desc = D3D12_COMMAND_QUEUE_DESC{
            Type: type_.d3dtype(),
            Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32,
            Flags: 0,
            NodeMask: 0,
        };

        let mut rawqueue: *mut ID3D12CommandQueue = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateCommandQueue(&desc, &ID3D12CommandQueue::uuidof(),
                                           &mut rawqueue as *mut *mut _ as *mut *mut c_void)
        };

        returnerrifwinerror!(hr, "Could not create command queue");

        Ok(SCommandQueue{queue: unsafe { ComPtr::from_raw(rawqueue) }})
    }
}

pub struct SSwapChain {
    buffercount: u32,
    swapchain: ComPtr<IDXGISwapChain4>,
    backbuffers: Vec<ComPtr<ID3D12Resource>>,
}

impl SD3D12 {
    pub fn createswapchain(&self, window: &SWindow, commandqueue: &mut SCommandQueue,
                           width: u32, height: u32) -> Result<SSwapChain, &'static str> {
        let buffercount = 2;

        let desc = DXGI_SWAP_CHAIN_DESC1{
            Width: width,
            Height: height,
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM, // $$$FRK(TODO): I have no idea why I'm picking this format
            Stereo: FALSE,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC{Count: 1, Quality: 0}, // $$$FRK(TODO): ???
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
                window.window,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut rawswapchain as *mut *mut _ as *mut *mut IDXGISwapChain1)
        };

        returnerrifwinerror!(hr, "Failed to create swap chain");

        let swapchain = unsafe { ComPtr::from_raw(rawswapchain) };
        match swapchain.cast::<IDXGISwapChain4>() {
            Ok(sc4) => {
                let mut backbuffers = Vec::with_capacity(2);
                for bbidx in 0..buffercount {
                    let mut rawbuf: *mut ID3D12Resource = ptr::null_mut();
                    let hn = unsafe {
                        sc4.GetBuffer(bbidx, &ID3D12Resource::uuidof(),
                                      &mut rawbuf as *mut *mut _ as *mut *mut c_void)
                    };

                    returnerrifwinerror!(hn, "Couldn't get ID3D12Resource for backbuffer from swapchain.");

                    backbuffers.push(unsafe { ComPtr::from_raw(rawbuf) });
                }

                Ok(SSwapChain{
                    buffercount: buffercount,
                    swapchain: sc4,
                    backbuffers: backbuffers,
                })
            },
            _ => Err("Swap chain could not be case to SwapChain4")
        }
    }
}

pub enum EDescriptorHeapType {
    ConstantBufferShaderResourceUnorderedAccess,
    Sampler,
    RenderTarget,
    DepthStencil,
}

pub struct SDescriptorHeap {
    type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
}

impl SDevice {
    pub fn createdescriptorheap(&self, type_: EDescriptorHeapType, numdescriptors: u32) -> Result<SDescriptorHeap, &'static str> {
        let d3dtype = match type_ {
            EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess => D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            EDescriptorHeapType::Sampler => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            EDescriptorHeapType::RenderTarget => D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            EDescriptorHeapType::DepthStencil => D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        };

        let desc = D3D12_DESCRIPTOR_HEAP_DESC{
            Type: d3dtype,
            NumDescriptors: numdescriptors,
            Flags: 0,
            NodeMask: 0,
        };

        let mut rawheap: *mut ID3D12DescriptorHeap = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(&desc, &ID3D12DescriptorHeap::uuidof(),
                                             &mut rawheap as *mut *mut _ as *mut *mut c_void)
        };

        returnerrifwinerror!(hr, "Failed to create descriptor heap");

        Ok(SDescriptorHeap{
            type_: type_,
            heap: unsafe { ComPtr::from_raw(rawheap) }
        })
    }

    pub fn initrendertargetviews(&self, swap: &SSwapChain, heap: &SDescriptorHeap) -> Result<(), &'static str> {
        match heap.type_ {
            EDescriptorHeapType::RenderTarget => {
                let descriptorsize = unsafe { self.device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) };
                let mut curdescriptorhandle = unsafe { heap.heap.GetCPUDescriptorHandleForHeapStart() };

                for backbuf in &swap.backbuffers {
                    unsafe {
                        self.device.CreateRenderTargetView(backbuf.as_raw(),
                                                           ptr::null(),
                                                           curdescriptorhandle);
                    };

                    curdescriptorhandle.ptr += descriptorsize as usize;
                    //curdescriptorhandle.Offset(descriptorsize);
                }

                Ok(())
            },
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap.")
        }
    }
}

pub struct SCommandAllocator {
    type_: ECommandListType,
    commandallocator: ComPtr<ID3D12CommandAllocator>,
}

pub struct SCommandList {
    commandlist: ComPtr<ID3D12GraphicsCommandList>,
}

impl SDevice {
    pub fn createcommandallocator(&self, type_: ECommandListType) -> Result<SCommandAllocator, &'static str> {
        let mut rawca: *mut ID3D12CommandAllocator = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandAllocator(type_.d3dtype(), &ID3D12CommandAllocator::uuidof(),
                                               &mut rawca as *mut *mut _ as *mut *mut c_void)
        };

        returnerrifwinerror!(hn, "Could not create command allocator.");

        Ok(SCommandAllocator{
            type_: type_,
            commandallocator: unsafe { ComPtr::from_raw(rawca) },
        })
    }

    pub fn createcommandlist(&self, allocator: &SCommandAllocator) -> Result<SCommandList, &'static str> {
        let mut rawcl: *mut ID3D12GraphicsCommandList = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandList(
                0,
                allocator.type_.d3dtype(),
                allocator.commandallocator.as_raw(),
                ptr::null_mut(),
                &ID3D12GraphicsCommandList::uuidof(),
                &mut rawcl as *mut *mut _ as *mut *mut c_void)
        };

        returnerrifwinerror!(hn, "Could not create command list.");

        Ok(SCommandList{
            commandlist: unsafe { ComPtr::from_raw(rawcl) },
        })

    }
}

pub struct SFence {
    fence: ComPtr<ID3D12Fence>,
}

impl SDevice {
    pub fn createfence(&self) -> Result<SFence, &'static str> {
        let mut rawf: *mut ID3D12Fence = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateFence(
                0, D3D12_FENCE_FLAG_NONE, &ID3D12Fence::uuidof(),
                &mut rawf as *mut *mut _ as *mut *mut c_void)
        };

        returnerrifwinerror!(hn, "Could not create fence.");

        Ok(SFence{
            fence: unsafe { ComPtr::from_raw(rawf) },
        })
    }
}

pub struct SEventHandle {
    event: winnt::HANDLE,
}

impl SWinAPI {
    pub fn createeventhandle() -> Result<SEventHandle, &'static str> {
        let event = unsafe { synchapi::CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null()) };

        if event == ntdef::NULL {
            return Err("Couldn't create event.");
        }

        Ok(SEventHandle{ event: event })
    }
}

impl SFence {
    pub fn waitforvalue(&self, val: u64, event: &SEventHandle, duration: u64) -> Result<(), &'static str> {
        if unsafe { self.fence.GetCompletedValue() } < val {
            let hn = unsafe { self.fence.SetEventOnCompletion(val, event.event) };
            returnerrifwinerror!(hn, "Could not set fence event on completion");
            unsafe { synchapi::WaitForSingleObject(event.event, duration as DWORD) };
        }

        Ok(())
    }
}

macro_rules! properror {
    ($result:expr) => {
        match $result {
            Ok(a) => a,
            Err(e) => { return Err(e); },
        }
    }
}

impl SCommandQueue {
    // -- $$$FRK(TODO): revisit this after I understand how I'm going to be using this fence
    pub fn pushsignal(&self, fence: &SFence, val: &mut u64) -> Result<u64, &'static str> {
        *val += 1;
        let hn = unsafe { self.queue.Signal(fence.fence.as_raw(), *val) };

        returnerrifwinerror!(hn, "Could not push signal.");

        Ok(*val)
    }

    pub fn flushgpublocking(&self, fence: &SFence, val: &mut u64, event: &SEventHandle) -> Result<(), &'static str> {
        let lastfencevalue = properror!(self.pushsignal(fence, val));
        properror!(fence.waitforvalue(lastfencevalue, event, <u64>::max_value()));
        Ok(())
    }
}

