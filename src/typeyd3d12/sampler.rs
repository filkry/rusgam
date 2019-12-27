use super::*;

pub enum EFilter {
    MinMagMipPoint,
    MinMagPointMipLinear,
    MinPointMagLinearMipPoint,
    MinPointMagMipLinear,
    MinLinearMagMipPoint,
    MinLinearMagPointMipLinear,
    MinMagLinearMipPoint,
    MinMagMipLinear,
    Anisotropic,
    ComparisonMinMagMipPoint,
    ComparisonMinMagPointMipLinear,
    ComparisonMinPointMagLinearMipPoint,
    ComparisonMinPointMagMipLinear,
    ComparisonMinLinearMagMipPoint,
    ComparisonMinLinearMagPointMipLinear,
    ComparisonMinMagLinearMipPoint,
    ComparisonMinMagMipLinear,
    ComparisonAnisotropic,
    MinimumMinMagMipPoint,
    MinimumMinMagPointMipLinear,
    MinimumMinPointMagLinearMipPoint,
    MinimumMinPointMagMipLinear,
    MinimumMinLinearMagMipPoint,
    MinimumMinLinearMagPointMipLinear,
    MinimumMinMagLinearMipPoint,
    MinimumMinMagMipLinear,
    MinimumAnisotropic,
    MaximumMinMagMipPoint,
    MaximumMinMagPointMipLinear,
    MaximumMinPointMagLinearMipPoint,
    MaximumMinPointMagMipLinear,
    MaximumMinLinearMagMipPoint,
    MaximumMinLinearMagPointMipLinear,
    MaximumMinMagLinearMipPoint,
    MaximumMinMagMipLinear,
    MaximumAnisotropic,
}

impl EFilter {
    pub fn d3dtype(&self) -> D3D12_FILTER {
        match self {
            Self::MinMagMipPoint => D3D12_FILTER_MIN_MAG_MIP_POINT,
            Self::MinMagPointMipLinear => D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR,
            Self::MinPointMagLinearMipPoint => D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
            Self::MinPointMagMipLinear => D3D12_FILTER_MIN_POINT_MAG_MIP_LINEAR,
            Self::MinLinearMagMipPoint => D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT,
            Self::MinLinearMagPointMipLinear => D3D12_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
            Self::MinMagLinearMipPoint => D3D12_FILTER_MIN_MAG_LINEAR_MIP_POINT,
            Self::MinMagMipLinear => D3D12_FILTER_MIN_MAG_MIP_LINEAR,
            Self::Anisotropic => D3D12_FILTER_ANISOTROPIC,
            Self::ComparisonMinMagMipPoint => D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT,
            Self::ComparisonMinMagPointMipLinear => {
                D3D12_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR
            }
            Self::ComparisonMinPointMagLinearMipPoint => {
                D3D12_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::ComparisonMinPointMagMipLinear => {
                D3D12_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR
            }
            Self::ComparisonMinLinearMagMipPoint => {
                D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT
            }
            Self::ComparisonMinLinearMagPointMipLinear => {
                D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::ComparisonMinMagLinearMipPoint => {
                D3D12_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT
            }
            Self::ComparisonMinMagMipLinear => D3D12_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR,
            Self::ComparisonAnisotropic => D3D12_FILTER_COMPARISON_ANISOTROPIC,
            Self::MinimumMinMagMipPoint => D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT,
            Self::MinimumMinMagPointMipLinear => D3D12_FILTER_MINIMUM_MIN_MAG_POINT_MIP_LINEAR,
            Self::MinimumMinPointMagLinearMipPoint => {
                D3D12_FILTER_MINIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::MinimumMinPointMagMipLinear => D3D12_FILTER_MINIMUM_MIN_POINT_MAG_MIP_LINEAR,
            Self::MinimumMinLinearMagMipPoint => D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_MIP_POINT,
            Self::MinimumMinLinearMagPointMipLinear => {
                D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::MinimumMinMagLinearMipPoint => D3D12_FILTER_MINIMUM_MIN_MAG_LINEAR_MIP_POINT,
            Self::MinimumMinMagMipLinear => D3D12_FILTER_MINIMUM_MIN_MAG_MIP_LINEAR,
            Self::MinimumAnisotropic => D3D12_FILTER_MINIMUM_ANISOTROPIC,
            Self::MaximumMinMagMipPoint => D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT,
            Self::MaximumMinMagPointMipLinear => D3D12_FILTER_MAXIMUM_MIN_MAG_POINT_MIP_LINEAR,
            Self::MaximumMinPointMagLinearMipPoint => {
                D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::MaximumMinPointMagMipLinear => D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_MIP_LINEAR,
            Self::MaximumMinLinearMagMipPoint => D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_MIP_POINT,
            Self::MaximumMinLinearMagPointMipLinear => {
                D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::MaximumMinMagLinearMipPoint => D3D12_FILTER_MAXIMUM_MIN_MAG_LINEAR_MIP_POINT,
            Self::MaximumMinMagMipLinear => D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_LINEAR,
            Self::MaximumAnisotropic => D3D12_FILTER_MAXIMUM_ANISOTROPIC,
        }
    }
}

pub enum ETextureAddressMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    MirrorOnce,
}

impl ETextureAddressMode {
    pub fn d3dtype(&self) -> D3D12_TEXTURE_ADDRESS_MODE {
        match self {
            Self::Wrap => D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            Self::Mirror => D3D12_TEXTURE_ADDRESS_MODE_MIRROR,
            Self::Clamp => D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            Self::Border => D3D12_TEXTURE_ADDRESS_MODE_BORDER,
            Self::MirrorOnce => D3D12_TEXTURE_ADDRESS_MODE_MIRROR_ONCE,
        }
    }
}

pub enum EStaticBorderColor {
    TransparentBlack,
    OpaqueBlack,
    OpaqueWhite,
}

impl EStaticBorderColor {
    pub fn d3dtype(&self) -> D3D12_STATIC_BORDER_COLOR {
        match self {
            Self::TransparentBlack => D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
            Self::OpaqueBlack => D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK,
            Self::OpaqueWhite => D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
        }
    }
}

pub struct SStaticSamplerDesc {
    pub filter: EFilter,
    pub address_u: ETextureAddressMode,
    pub address_v: ETextureAddressMode,
    pub address_w: ETextureAddressMode,
    pub mip_lod_bias: f32,
    pub max_anisotropy: u32,
    pub comparison_func: EComparisonFunc,
    pub border_color: EStaticBorderColor,
    pub min_lod: f32,
    pub max_lod: f32,
    pub shader_register: u32,
    pub register_space: u32,
    pub shader_visibility: EShaderVisibility,
}

impl SStaticSamplerDesc {
    pub fn d3dtype(&self) -> D3D12_STATIC_SAMPLER_DESC {
        D3D12_STATIC_SAMPLER_DESC {
            Filter: self.filter.d3dtype(),
            AddressU: self.address_u.d3dtype(),
            AddressV: self.address_v.d3dtype(),
            AddressW: self.address_w.d3dtype(),
            MipLODBias: self.mip_lod_bias,
            MaxAnisotropy: self.max_anisotropy,
            ComparisonFunc: self.comparison_func.d3dtype(),
            BorderColor: self.border_color.d3dtype(),
            MinLOD: self.min_lod,
            MaxLOD: self.max_lod,
            ShaderRegister: self.shader_register,
            RegisterSpace: self.register_space,
            ShaderVisibility: self.shader_visibility.d3dtype(),
        }
    }
}
