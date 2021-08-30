use super::*;

use bitflags::*;
use std::convert::From;

pub struct SDXGISampleDesc {
    pub count: u32,
    pub quality: u32,
}

impl SDXGISampleDesc {
    pub fn d3dtype(&self) -> win::DXGI_SAMPLE_DESC {
        win::DXGI_SAMPLE_DESC {
            Count: self.count,
            Quality: self.quality,
        }
    }
}

impl From<win::DXGI_SAMPLE_DESC> for SDXGISampleDesc {
    fn from(desc: win::DXGI_SAMPLE_DESC) -> Self {
        Self {
            count: desc.Count,
            quality: desc.Quality,
        }
    }
}

bitflags! {
    pub struct SDXGIUsageFlags: u32{
        const BACK_BUFFER = win::DXGI_USAGE_BACK_BUFFER;
        const DISCARD_ON_PRESENT = win::DXGI_USAGE_DISCARD_ON_PRESENT;
        const READ_ONLY = win::DXGI_USAGE_READ_ONLY;
        const RENDER_TARGET_OUTPUT = win::DXGI_USAGE_RENDER_TARGET_OUTPUT;
        const SHADER_INPUT = win::DXGI_USAGE_SHADER_INPUT;
        const SHARED = win::DXGI_USAGE_SHARED;
        const UNORDERED_ACCESS = win::DXGI_USAGE_UNORDERED_ACCESS;
    }
}

pub enum EDXGIScaling {
    Stretch,
    None,
    AspectRatioStretch,
}

impl EDXGIScaling {
    pub fn d3dtype(&self) -> win::DXGI_SCALING {
        match self {
            Self::Stretch => win::DXGI_SCALING_STRETCH,
            Self::None => win::DXGI_SCALING_NONE,
            Self::AspectRatioStretch => win::DXGI_SCALING_ASPECT_RATIO_STRETCH,
        }
    }
}

impl From<win::DXGI_SCALING> for EDXGIScaling {
    fn from(raw: win::DXGI_SCALING) -> Self {
        match raw {
            win::DXGI_SCALING_STRETCH => Self::Stretch,
            win::DXGI_SCALING_NONE => Self::None,
            win::DXGI_SCALING_ASPECT_RATIO_STRETCH => Self::AspectRatioStretch,
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
    pub fn d3dtype(&self) -> win::DXGI_SWAP_EFFECT {
        match self {
            Self::Discard => win::DXGI_SWAP_EFFECT_DISCARD,
            Self::Sequential => win::DXGI_SWAP_EFFECT_SEQUENTIAL,
            Self::FlipSequential => win::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            Self::FlipDiscard => win::DXGI_SWAP_EFFECT_FLIP_DISCARD,
        }
    }
}

impl From<win::DXGI_SWAP_EFFECT> for EDXGISwapEffect {
    fn from(effect: win::DXGI_SWAP_EFFECT) -> Self {
        match effect {
            win::DXGI_SWAP_EFFECT_DISCARD => Self::Discard,
            win::DXGI_SWAP_EFFECT_SEQUENTIAL => Self::Sequential,
            win::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL => Self::FlipSequential,
            win::DXGI_SWAP_EFFECT_FLIP_DISCARD => Self::FlipDiscard,
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
    pub fn d3dtype(&self) -> win::DXGI_ALPHA_MODE {
        match self {
            Self::Unspecified => win::DXGI_ALPHA_MODE_UNSPECIFIED,
            Self::Premultiplied => win::DXGI_ALPHA_MODE_PREMULTIPLIED,
            Self::Straight => win::DXGI_ALPHA_MODE_STRAIGHT,
            Self::Ignore => win::DXGI_ALPHA_MODE_IGNORE,
            Self::ForceDWord => win::DXGI_ALPHA_MODE_FORCE_DWORD,
        }
    }
}

impl From<win::DXGI_ALPHA_MODE> for EDXGIAlphaMode {
    fn from(mode: win::DXGI_ALPHA_MODE) -> Self {
        match mode {
            win::DXGI_ALPHA_MODE_UNSPECIFIED => Self::Unspecified,
            win::DXGI_ALPHA_MODE_PREMULTIPLIED => Self::Premultiplied,
            win::DXGI_ALPHA_MODE_STRAIGHT => Self::Straight,
            win::DXGI_ALPHA_MODE_IGNORE => Self::Ignore,
            win::DXGI_ALPHA_MODE_FORCE_DWORD => Self::ForceDWord,
            _ => panic!("Bad alpha mode"),
        }
    }
}

bitflags! {
    pub struct SDXGISwapChainFlags: i32 {
        const NONPREROTATED = win::DXGI_SWAP_CHAIN_FLAG_NONPREROTATED.0;
        const ALLOW_MODE_SWITCH = win::DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0;
        const GDI_COMPATIBLE = win::DXGI_SWAP_CHAIN_FLAG_GDI_COMPATIBLE.0;
        const RESTRICTED_CONTENT = win::DXGI_SWAP_CHAIN_FLAG_RESTRICTED_CONTENT.0;
        const RESTRICT_SHARED_RESOURCE_DRIVER = win::DXGI_SWAP_CHAIN_FLAG_RESTRICT_SHARED_RESOURCE_DRIVER.0;
        const DISPLAY_ONLY = win::DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY.0;
        const FRAME_LATENCY_WAITABLE_OBJECT = win::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0;
        const FOREGROUND_LAYER = win::DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER.0;
        const FULLSCREEN_VIDEO = win::DXGI_SWAP_CHAIN_FLAG_FULLSCREEN_VIDEO.0;
        const YUV_VIDEO = win::DXGI_SWAP_CHAIN_FLAG_YUV_VIDEO.0;
        const HW_PROTECTED = win::DXGI_SWAP_CHAIN_FLAG_HW_PROTECTED.0;
        const ALLOW_TEARING = win::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0;
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
    pub fn d3dtype(&self) -> win::DXGI_SWAP_CHAIN_DESC1 {
        win::DXGI_SWAP_CHAIN_DESC1 {
            Width: self.width,
            Height: self.height,
            Format: EDXGIFormat::R8G8B8A8UNorm.d3dtype(), // $$$FRK(TODO): I have no idea why I'm picking this format
            Stereo: win::BOOL::from(self.stereo),
            SampleDesc: self.sample_desc.d3dtype(),
            BufferUsage: self.buffer_usage.bits(),
            BufferCount: self.buffer_count,
            Scaling: self.scaling.d3dtype(),
            SwapEffect: self.swap_effect.d3dtype(),
            AlphaMode: self.alpha_mode.d3dtype(),
            Flags: self.flags.bits() as u32,
        }
    }
}

impl From<win::DXGI_SWAP_CHAIN_DESC1> for SSwapChainDesc {
    fn from(desc: win::DXGI_SWAP_CHAIN_DESC1) -> Self {
        Self {
            width: desc.Width,
            height: desc.Height,
            format: EDXGIFormat::from(desc.Format),
            stereo: desc.Stereo.as_bool(),
            sample_desc: SDXGISampleDesc::from(desc.SampleDesc),
            buffer_usage: SDXGIUsageFlags::from_bits(desc.BufferUsage).unwrap(),
            buffer_count: desc.BufferCount,
            scaling: EDXGIScaling::from(desc.Scaling),
            swap_effect: EDXGISwapEffect::from(desc.SwapEffect),
            alpha_mode: EDXGIAlphaMode::from(desc.AlphaMode),
            flags: SDXGISwapChainFlags::from_bits(desc.Flags as i32).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct SSwapChain {
    swapchain: win::IDXGISwapChain4,
}

impl SSwapChain {
    pub unsafe fn new_from_raw(raw: win::IDXGISwapChain4) -> Self {
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
        let hn = unsafe {
            self.swapchain.GetBuffer::<win::ID3D12Resource>(
                idx as u32,
            )
        };

        returnerrifwinerror!(
            hn,
            "Couldn't get ID3D12Resource for backbuffer from swapchain."
        );

        Ok(unsafe { SResource::new_from_raw(hn.expect("checked err above")) })
    }

    pub fn getdesc(&self) -> Result<SSwapChainDesc, &'static str> {
        unsafe {
            //let mut desc: win::DXGI_SWAP_CHAIN_DESC1 = mem::zeroed();
            let hr = self.swapchain.GetDesc1();
            returnerrifwinerror!(hr, "Couldn't get swap chain desc.");
            Ok(SSwapChainDesc::from(hr.expect("checked err above")))
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
                olddesc.flags.bits() as u32,
            );
            returnerrifwinerror!(hr, "Couldn't resize buffers.");
        }
        Ok(())
    }
}