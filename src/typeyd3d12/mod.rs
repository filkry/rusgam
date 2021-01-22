#![allow(dead_code)]

// -- FRK(TODO): actually use the relevant windows error
macro_rules! returnerrifwinerror {
    ($hn:expr, $err:expr) => {
        if($hn.is_err()) {
            return Err($err);
        }
    };
}

mod adapter;
mod commandallocator;
mod commandlist;
mod commandqueue;
mod debuginterface;
mod descriptor;
mod device;
mod factory;
mod fence;
mod heap;
mod infoqueue;
mod pipelinestate;
mod resource;
mod rootsignature;
mod sampler;
mod shader;
mod swapchain;
mod view;

use crate::safewindows;
use crate::enumflags::{TEnumFlags32, SEnumFlags32};

use std::{mem, ptr};
use std::convert::From;

use arrayvec::ArrayVec;
use win;

pub use self::adapter::SAdapter1;
pub use self::adapter::SAdapter4;
pub use self::commandallocator::*;
pub use self::commandlist::*;
pub use self::commandqueue::*;
pub use self::debuginterface::*;
pub use self::descriptor::*;
pub use self::device::*;
pub use self::factory::SFactory;
pub use self::fence::SFence;
pub use self::heap::*;
pub use self::infoqueue::SInfoQueue;
pub use self::pipelinestate::*;
pub use self::resource::*;
pub use self::rootsignature::*;
pub use self::sampler::*;
pub use self::shader::*;
pub use self::swapchain::*;
pub use self::view::*;

pub struct SBarrier {
    barrier: win::D3D12_RESOURCE_BARRIER,
}

pub struct SScissorRects {
    rects: ArrayVec<[SRect; 16]>,

    d3drects: ArrayVec<[win::RECT; 16]>,
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
    raw: win::D3D12_GPU_VIRTUAL_ADDRESS,
}

impl SGPUVirtualAddress {
    pub fn raw(&self) -> win::D3D12_GPU_VIRTUAL_ADDRESS {
        self.raw
    }

    pub fn add(&self, offset: usize) -> SGPUVirtualAddress {
        SGPUVirtualAddress {
            raw: (self.raw + (offset as u64)),
        }
    }
}

pub struct SViewport {
    viewport: win::D3D12_VIEWPORT,
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
            viewport: win::D3D12_VIEWPORT {
                TopLeftX: topleftx,
                TopLeftY: toplefty,
                Width: width,
                Height: height,
                MinDepth: mindepth.unwrap_or(win::D3D12_MIN_DEPTH),
                MaxDepth: maxdepth.unwrap_or(win::D3D12_MAX_DEPTH),
            },
        }
    }
}

pub type SRect = safewindows::SRect;

impl SRect {
    pub fn d3dtype(&self) -> win::D3D12_RECT {
        win::D3D12_RECT{
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
    R32G32B32A32Float,
    R32G32B32Float,
    R32G32Float,
    R32Typeless,
    R32Float,
    D32Float,
    R8G8B8A8UNorm,
    R16UINT,
    R32UINT,
}

impl EDXGIFormat {
    pub fn d3dtype(&self) -> win::DXGI_FORMAT {
        match self {
            Self::Unknown => win::DXGI_FORMAT_UNKNOWN,
            Self::R32G32B32A32Typeless => win::DXGI_FORMAT_R32G32B32A32_TYPELESS,
            Self::R32G32B32A32Float => win::DXGI_FORMAT_R32G32B32A32_FLOAT,
            Self::R32G32B32Float => win::DXGI_FORMAT_R32G32B32_FLOAT,
            Self::R32G32Float => win::DXGI_FORMAT_R32G32_FLOAT,
            Self::D32Float => win::DXGI_FORMAT_D32_FLOAT,
            Self::R32Float => win::DXGI_FORMAT_R32_FLOAT,
            Self::R32Typeless => win::DXGI_FORMAT_R32_TYPELESS,
            Self::R8G8B8A8UNorm => win::DXGI_FORMAT_R8G8B8A8_UNORM,
            Self::R16UINT => win::DXGI_FORMAT_R16_UINT,
            Self::R32UINT => win::DXGI_FORMAT_R32_UINT,
        }
    }
}

impl From<win::DXGI_FORMAT> for EDXGIFormat {
    fn from(format: win::DXGI_FORMAT) -> Self {
        match format {
            win::DXGI_FORMAT_UNKNOWN => Self::Unknown,
            win::DXGI_FORMAT_R32G32B32A32_TYPELESS => Self::R32G32B32A32Typeless,
            win::DXGI_FORMAT_R32G32B32_FLOAT => Self::R32G32B32Float,
            win::DXGI_FORMAT_R32G32_FLOAT => Self::R32G32Float,
            win::DXGI_FORMAT_D32_FLOAT => Self::D32Float,
            win::DXGI_FORMAT_R32_FLOAT => Self::R32Float,
            win::DXGI_FORMAT_R32_TYPELESS => Self::R32Typeless,
            win::DXGI_FORMAT_R8G8B8A8_UNORM => Self::R8G8B8A8UNorm,
            win::DXGI_FORMAT_R16_UINT => Self::R16UINT,
            _ => {
                panic!("Unimplemented type");
            }
        }
    }
}

pub struct SBlob {
    raw: win::ID3DBlob,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDepthWriteMask {
    Zero,
    All,
}

impl EDepthWriteMask {
    pub fn d3dtype(&self) -> win::D3D12_DEPTH_WRITE_MASK {
        match self {
            Self::Zero => win::D3D12_DEPTH_WRITE_MASK_ZERO,
            Self::All => win::D3D12_DEPTH_WRITE_MASK_ALL,
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

pub struct SRange {
    begin: usize,
    end: usize,
}

impl SRange {
    pub fn d3dtype(&self) -> win::D3D12_RANGE {
        win::D3D12_RANGE {
            Begin: self.begin,
            End: self.end,
        }
    }
}

impl EComparisonFunc {
    pub fn d3dtype(&self) -> win::D3D12_COMPARISON_FUNC {
        match self {
            Self::Never => win::D3D12_COMPARISON_FUNC_NEVER,
            Self::Less => win::D3D12_COMPARISON_FUNC_LESS,
            Self::Equal => win::D3D12_COMPARISON_FUNC_EQUAL,
            Self::LessEqual => win::D3D12_COMPARISON_FUNC_LESS_EQUAL,
            Self::Greater => win::D3D12_COMPARISON_FUNC_GREATER,
            Self::NotEqual => win::D3D12_COMPARISON_FUNC_NOT_EQUAL,
            Self::GreaterEqual => win::D3D12_COMPARISON_FUNC_GREATER_EQUAL,
            Self::Always => win::D3D12_COMPARISON_FUNC_ALWAYS,
        }
    }
}

// -- $$$FRK(FUTURE WORK): unsupported:
// --    + pDefines
// --    + pInclude
// --    + flags2
pub fn d3dcompilefromfile(
    file: &str,
    entrypoint: &str,
    target: &str,
    flags1: SCompile,
) -> Result<SBlob, &'static str> {
    let mut rawcodeblob: Option<win::ID3DBlob> = None;
    let mut errormsgsblob: Option<win::ID3DBlob> = None;

    let hr = unsafe {
        win::D3DCompileFromFile(
            file,
            ptr::null_mut(),
            None,
            entrypoint,
            target,
            flags1.rawtype(),
            0,
            &mut rawcodeblob,
            &mut errormsgsblob,
        )
    };

    returnerrifwinerror!(hr, "failed to compile shader");
    // -- $$$FRK(FUTURE WORK): use error messages blob

    Ok(SBlob {
        raw: rawcodeblob.expect("checked err above"),
    })
}

pub fn read_file_to_blob(file: &str) -> Result<SBlob, &'static str> {
    let hr = unsafe { win::D3DReadFileToBlob(file) };

    returnerrifwinerror!(hr, "failed to load shader");

    Ok(SBlob {
        raw: hr.expect("checked err above"),
    })
}
