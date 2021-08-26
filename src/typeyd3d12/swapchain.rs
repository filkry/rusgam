use super::*;

use bitflags::*;
use std::convert::From;

pub struct SDXGISampleDesc {
    pub count: u32,
    pub quality: u32,
}

impl SDXGISampleDesc {
    pub fn d3dtype(&self) -> DXGI_SAMPLE_DESC {
        DXGI_SAMPLE_DESC {
            Count: self.count,
            Quality: self.quality,
        }
    }
}

impl From<DXGI_SAMPLE_DESC> for SDXGISampleDesc {
    fn from(desc: DXGI_SAMPLE_DESC) -> Self {
        Self {
            count: desc.Count,
            quality: desc.Quality,
        }
    }
}

bitflags! {
    pub struct SDXGIUsageFlags: DXGI_USAGE {
        const BACK_BUFFER = DXGI_USAGE_BACK_BUFFER;
        const DISCARD_ON_PRESENT = DXGI_USAGE_DISCARD_ON_PRESENT;
        const READ_ONLY = DXGI_USAGE_READ_ONLY;
        const RENDER_TARGET_OUTPUT = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        const SHADER_INPUT = DXGI_USAGE_SHADER_INPUT;
        const SHARED = DXGI_USAGE_SHARED;
        const UNORDERED_ACCESS = DXGI_USAGE_UNORDERED_ACCESS;
    }
}

pub enum EDXGIScaling {
    Stretch,
    None,
    AspectRatioStretch,
}

impl EDXGIScaling {
    pub fn d3dtype(&self) -> DXGI_SCALING {
        match self {
            Self::Stretch => DXGI_SCALING_STRETCH,
            Self::None => DXGI_SCALING_NONE,
            Self::AspectRatioStretch => DXGI_SCALING_ASPECT_RATIO_STRETCH,
        }
    }
}

impl From<DXGI_SCALING> for EDXGIScaling {
    fn from(raw: DXGI_SCALING) -> Self {
        match raw {
            DXGI_SCALING_STRETCH => Self::Stretch,
            DXGI_SCALING_NONE => Self::None,
            DXGI_SCALING_ASPECT_RATIO_STRETCH => Self::AspectRatioStretch,
            _ => panic!("Bad data"),
        }
    }
}

pub enum EDXGISwapEffect {
    Discard,
    Sequential,
    FlipSequential,
    FlipDiscard,
}

impl EDXGISwapEffect {
    pub fn d3dtype(&self) -> DXGI_SWAP_EFFECT {
        match self {
            Self::Discard => DXGI_SWAP_EFFECT_DISCARD,
            Self::Sequential => DXGI_SWAP_EFFECT_SEQUENTIAL,
            Self::FlipSequential => DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            Self::FlipDiscard => DXGI_SWAP_EFFECT_FLIP_DISCARD,
        }
    }
}

impl From<DXGI_SWAP_EFFECT> for EDXGISwapEffect {
    fn from(effect: DXGI_SWAP_EFFECT) -> Self {
        match effect {
            DXGI_SWAP_EFFECT_DISCARD => Self::Discard,
            DXGI_SWAP_EFFECT_SEQUENTIAL => Self::Sequential,
            DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL => Self::FlipSequential,
            DXGI_SWAP_EFFECT_FLIP_DISCARD => Self::FlipDiscard,
            _ => panic!("Bad swap effect value"),
        }
    }
}

pub enum EDXGIAlphaMode {
    Unspecified,
    Premultiplied,
    Straight,
    Ignore,
    ForceDWord,
}

impl EDXGIAlphaMode {
    pub fn d3dtype(&self) -> DXGI_ALPHA_MODE {
        match self {
            Self::Unspecified => DXGI_ALPHA_MODE_UNSPECIFIED,
            Self::Premultiplied => DXGI_ALPHA_MODE_PREMULTIPLIED,
            Self::Straight => DXGI_ALPHA_MODE_STRAIGHT,
            Self::Ignore => DXGI_ALPHA_MODE_IGNORE,
            Self::ForceDWord => DXGI_ALPHA_MODE_FORCE_DWORD,
        }
    }
}

impl From<DXGI_ALPHA_MODE> for EDXGIAlphaMode {
    fn from(mode: DXGI_ALPHA_MODE) -> Self {
        match mode {
            DXGI_ALPHA_MODE_UNSPECIFIED => Self::Unspecified,
            DXGI_ALPHA_MODE_PREMULTIPLIED => Self::Premultiplied,
            DXGI_ALPHA_MODE_STRAIGHT => Self::Straight,
            DXGI_ALPHA_MODE_IGNORE => Self::Ignore,
            DXGI_ALPHA_MODE_FORCE_DWORD => Self::ForceDWord,
            _ => panic!("Bad alpha mode"),
        }
    }
}

bitflags! {
    pub struct SDXGISwapChainFlags: DXGI_SWAP_CHAIN_FLAG {
        const NONPREROTATED = DXGI_SWAP_CHAIN_FLAG_NONPREROTATED;
        const ALLOW_MODE_SWITCH = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
        const GDI_COMPATIBLE = DXGI_SWAP_CHAIN_FLAG_GDI_COMPATIBLE;
        const RESTRICTED_CONTENT = DXGI_SWAP_CHAIN_FLAG_RESTRICTED_CONTENT;
        const RESTRICT_SHARED_RESOURCE_DRIVER = DXGI_SWAP_CHAIN_FLAG_RESTRICT_SHARED_RESOURCE_DRIVER;
        const DISPLAY_ONLY = DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY;
        const FRAME_LATENCY_WAITABLE_OBJECT = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT;
        const FOREGROUND_LAYER = DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER;
        const FULLSCREEN_VIDEO = DXGI_SWAP_CHAIN_FLAG_FULLSCREEN_VIDEO;
        const YUV_VIDEO = DXGI_SWAP_CHAIN_FLAG_YUV_VIDEO;
        const HW_PROTECTED = DXGI_SWAP_CHAIN_FLAG_HW_PROTECTED;
        const ALLOW_TEARING = DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING;
        //const RESTRICTED_TO_ALL_HOLOGRAPHIC_DISPLAYS = DXGI_SWAP_CHAIN_FLAG_RESTRICTED_TO_ALL_HOLOGRAPHIC_DISPLAY;
    }
}

pub struct SSwapChainDesc {
    pub width: u32,
    pub height: u32,
    pub format: EDXGIFormat,
    pub stereo: bool,
    pub sample_desc: SDXGISampleDesc,
    pub buffer_usage: SDXGIUsageFlags,
    pub buffer_count: u32,
    pub scaling: EDXGIScaling,
    pub swap_effect: EDXGISwapEffect,
    pub alpha_mode: EDXGIAlphaMode,
    pub flags: SDXGISwapChainFlags,
}

impl SSwapChainDesc {
    pub fn d3dtype(&self) -> DXGI_SWAP_CHAIN_DESC1 {
        DXGI_SWAP_CHAIN_DESC1 {
            Width: self.width,
            Height: self.height,
            Format: EDXGIFormat::R8G8B8A8UNorm.d3dtype(), // $$$FRK(TODO): I have no idea why I'm picking this format
            Stereo: if self.stereo { TRUE } else { FALSE },
            SampleDesc: self.sample_desc.d3dtype(),
            BufferUsage: self.buffer_usage.bits(),
            BufferCount: self.buffer_count,
            Scaling: self.scaling.d3dtype(),
            SwapEffect: self.swap_effect.d3dtype(),
            AlphaMode: self.alpha_mode.d3dtype(),
            Flags: self.flags.bits(),
        }
    }
}

impl From<DXGI_SWAP_CHAIN_DESC1> for SSwapChainDesc {
    fn from(desc: DXGI_SWAP_CHAIN_DESC1) -> Self {
        Self {
            width: desc.Width,
            height: desc.Height,
            format: EDXGIFormat::from(desc.Format),
            stereo: if desc.Stereo == TRUE { true } else { false },
            sample_desc: SDXGISampleDesc::from(desc.SampleDesc),
            buffer_usage: SDXGIUsageFlags::from_bits(desc.BufferUsage).unwrap(),
            buffer_count: desc.BufferCount,
            scaling: EDXGIScaling::from(desc.Scaling),
            swap_effect: EDXGISwapEffect::from(desc.SwapEffect),
            alpha_mode: EDXGIAlphaMode::from(desc.AlphaMode),
            flags: SDXGISwapChainFlags::from_bits(desc.Flags).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct SSwapChain {
    swapchain: ComPtr<IDXGISwapChain4>,
}

impl SSwapChain {
    pub unsafe fn new_from_raw(raw: ComPtr<IDXGISwapChain4>) -> Self {
        Self { swapchain: raw }
    }

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

        Ok(unsafe { SResource::new_from_raw(ComPtr::from_raw(rawbuf)) })
    }

    pub fn getdesc(&self) -> Result<SSwapChainDesc, &'static str> {
        unsafe {
            let mut desc: DXGI_SWAP_CHAIN_DESC1 = mem::zeroed();
            let hr = self.swapchain.GetDesc1(&mut desc as *mut _);
            returnerrifwinerror!(hr, "Couldn't get swap chain desc.");
            Ok(SSwapChainDesc::from(desc))
        }
    }

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
                olddesc.format.d3dtype(),
                olddesc.flags.bits(),
            );
            returnerrifwinerror!(hr, "Couldn't resize buffers.");
        }
        Ok(())
    }
}