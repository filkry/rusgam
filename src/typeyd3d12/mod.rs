#![allow(dead_code)]

macro_rules! returnerrifwinerror {
    ($hn:expr, $err:expr) => {
        if !winerror::SUCCEEDED($hn) {
            return Err($err);
        }
    };
}

mod debuginterface;
mod factory;
mod adapter;
mod resource;
mod device;
mod infoqueue;
mod commandlist;
mod commandqueue;
mod commandallocator;
mod swapchain;
mod descriptor;
mod heap;
mod fence;
mod view;
mod rootsignature;
mod pipelinestate;
mod shader;

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

pub trait TD3DFlags32 {
    type TD3DType: std::convert::Into<u32> + std::convert::From<u32> + Copy + Clone;

    fn d3dtype(&self) -> Self::TD3DType;
}

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

pub use self::debuginterface::SDebugInterface;
pub use self::factory::SFactory;
pub use self::adapter::SAdapter1;
pub use self::adapter::SAdapter4;
pub use self::resource::*;
pub use self::device::*;
pub use self::infoqueue::SInfoQueue;
pub use self::commandlist::*;
pub use self::commandqueue::*;
pub use self::commandallocator::*;
pub use self::swapchain::*;
pub use self::descriptor::*;
pub use self::heap::*;
pub use self::fence::SFence;
pub use self::view::*;
pub use self::rootsignature::*;
pub use self::pipelinestate::*;
pub use self::shader::*;

pub struct SBarrier {
    barrier: D3D12_RESOURCE_BARRIER,
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

pub struct SGPUVirtualAddress {
    raw: D3D12_GPU_VIRTUAL_ADDRESS,
}

impl SGPUVirtualAddress {
    pub fn raw(&self) -> D3D12_GPU_VIRTUAL_ADDRESS {
        self.raw
    }
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

// -- $$$FRK(TODO): should just be D3DRECT
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
    R16UINT,
}

impl EDXGIFormat {
    pub fn d3dtype(&self) -> dxgiformat::DXGI_FORMAT {
        match self {
            Self::Unknown => dxgiformat::DXGI_FORMAT_UNKNOWN,
            Self::R32G32B32A32Typeless => dxgiformat::DXGI_FORMAT_R32G32B32A32_TYPELESS,
            Self::R32G32B32Float => dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            Self::D32Float => dxgiformat::DXGI_FORMAT_D32_FLOAT,
            Self::R8G8B8A8UNorm => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Self::R16UINT => dxgiformat::DXGI_FORMAT_R16_UINT,
        }
    }
}

pub struct SBlob {
    raw: ComPtr<d3dcommon::ID3DBlob>,
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
