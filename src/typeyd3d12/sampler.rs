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
    pub fn d3dtype(&self) -> win::D3D12_FILTER {
        match self {
            Self::MinMagMipPoint => win::D3D12_FILTER_MIN_MAG_MIP_POINT,
            Self::MinMagPointMipLinear => win::D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR,
            Self::MinPointMagLinearMipPoint => win::D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
            Self::MinPointMagMipLinear => win::D3D12_FILTER_MIN_POINT_MAG_MIP_LINEAR,
            Self::MinLinearMagMipPoint => win::D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT,
            Self::MinLinearMagPointMipLinear => win::D3D12_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
            Self::MinMagLinearMipPoint => win::D3D12_FILTER_MIN_MAG_LINEAR_MIP_POINT,
            Self::MinMagMipLinear => win::D3D12_FILTER_MIN_MAG_MIP_LINEAR,
            Self::Anisotropic => win::D3D12_FILTER_ANISOTROPIC,
            Self::ComparisonMinMagMipPoint => win::D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT,
            Self::ComparisonMinMagPointMipLinear => {
                win::D3D12_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR
            }
            Self::ComparisonMinPointMagLinearMipPoint => {
                win::D3D12_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::ComparisonMinPointMagMipLinear => {
                win::D3D12_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR
            }
            Self::ComparisonMinLinearMagMipPoint => {
                win::D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT
            }
            Self::ComparisonMinLinearMagPointMipLinear => {
                win::D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::ComparisonMinMagLinearMipPoint => {
                win::D3D12_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT
            }
            Self::ComparisonMinMagMipLinear => win::D3D12_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR,
            Self::ComparisonAnisotropic => win::D3D12_FILTER_COMPARISON_ANISOTROPIC,
            Self::MinimumMinMagMipPoint => win::D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT,
            Self::MinimumMinMagPointMipLinear => win::D3D12_FILTER_MINIMUM_MIN_MAG_POINT_MIP_LINEAR,
            Self::MinimumMinPointMagLinearMipPoint => {
                win::D3D12_FILTER_MINIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::MinimumMinPointMagMipLinear => win::D3D12_FILTER_MINIMUM_MIN_POINT_MAG_MIP_LINEAR,
            Self::MinimumMinLinearMagMipPoint => win::D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_MIP_POINT,
            Self::MinimumMinLinearMagPointMipLinear => {
                win::D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::MinimumMinMagLinearMipPoint => win::D3D12_FILTER_MINIMUM_MIN_MAG_LINEAR_MIP_POINT,
            Self::MinimumMinMagMipLinear => win::D3D12_FILTER_MINIMUM_MIN_MAG_MIP_LINEAR,
            Self::MinimumAnisotropic => win::D3D12_FILTER_MINIMUM_ANISOTROPIC,
            Self::MaximumMinMagMipPoint => win::D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT,
            Self::MaximumMinMagPointMipLinear => win::D3D12_FILTER_MAXIMUM_MIN_MAG_POINT_MIP_LINEAR,
            Self::MaximumMinPointMagLinearMipPoint => {
                win::D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            Self::MaximumMinPointMagMipLinear => win::D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_MIP_LINEAR,
            Self::MaximumMinLinearMagMipPoint => win::D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_MIP_POINT,
            Self::MaximumMinLinearMagPointMipLinear => {
                win::D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            Self::MaximumMinMagLinearMipPoint => win::D3D12_FILTER_MAXIMUM_MIN_MAG_LINEAR_MIP_POINT,
            Self::MaximumMinMagMipLinear => win::D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_LINEAR,
            Self::MaximumAnisotropic => win::D3D12_FILTER_MAXIMUM_ANISOTROPIC,
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
    pub fn d3dtype(&self) -> win::D3D12_TEXTURE_ADDRESS_MODE {
        match self {
            Self::Wrap => win::D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            Self::Mirror => win::D3D12_TEXTURE_ADDRESS_MODE_MIRROR,
            Self::Clamp => win::D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
            Self::Border => win::D3D12_TEXTURE_ADDRESS_MODE_BORDER,
            Self::MirrorOnce => win::D3D12_TEXTURE_ADDRESS_MODE_MIRROR_ONCE,
        }
    }
}

pub enum EStaticBorderColor {
    TransparentBlack,
    OpaqueBlack,
    OpaqueWhite,
}

impl EStaticBorderColor {
    pub fn d3dtype(&self) -> win::D3D12_STATIC_BORDER_COLOR {
        match self {
            Self::TransparentBlack => win::D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
            Self::OpaqueBlack => win::D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK,
            Self::OpaqueWhite => win::D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
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
    pub fn d3dtype(&self) -> win::D3D12_STATIC_SAMPLER_DESC {
        win::D3D12_STATIC_SAMPLER_DESC {
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
