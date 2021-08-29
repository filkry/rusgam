use super::*;

use arrayvec::{ArrayVec};
use std::mem::{size_of};

#[repr(C)]
pub struct SPipelineStateStreamRootSignature<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: &'a win::ID3D12RootSignature,
}

impl<'a> SPipelineStateStreamRootSignature<'a> {
    pub fn create(src: &'a SRootSignature) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::RootSignature.d3dtype(),
            value: src.raw().raw.deref(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamVertexShader<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_SHADER_BYTECODE,
    phantom: PhantomData<&'a t12::SShaderBytecode>,
}

impl<'a> SPipelineStateStreamVertexShader<'a> {
    pub fn create(shader_bytecode: &'a t12::SShaderBytecode) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::VS.d3dtype(),
            value: unsafe { shader_bytecode.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamPixelShader<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_SHADER_BYTECODE,
    phantom: PhantomData<&'a t12::SShaderBytecode>,
}

impl<'a> SPipelineStateStreamPixelShader<'a> {
    pub fn create(shader_bytecode: &'a t12::SShaderBytecode) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::PS.d3dtype(),
            value: unsafe { shader_bytecode.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamComputeShader<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_SHADER_BYTECODE,
    phantom: PhantomData<&'a t12::SShaderBytecode>,
}

impl<'a> SPipelineStateStreamComputeShader<'a> {
    pub fn create(shader_bytecode: &'a t12::SShaderBytecode) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::CS.d3dtype(),
            value: unsafe { shader_bytecode.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C, align(8))]
pub struct SPipelineStateStreamBlendDesc {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_BLEND_DESC,
}

impl SPipelineStateStreamBlendDesc {
    pub fn create(blend_desc: t12::SBlendDesc) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        // -- result keeps pointer to input!
        Self {
            type_: t12::EPipelineStateSubobjectType::Blend.d3dtype(),
            value: blend_desc.d3dtype(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamInputLayout<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_INPUT_LAYOUT_DESC,
    phantom: PhantomData<&'a t12::SInputLayoutDesc>,
}

impl<'a> SPipelineStateStreamInputLayout<'a> {
    pub fn create(input_layout: &'a mut t12::SInputLayoutDesc) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::InputLayout.d3dtype(),
            value: unsafe { input_layout.d3dtype() },
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamPrimitiveTopology {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_PRIMITIVE_TOPOLOGY_TYPE,
}

impl SPipelineStateStreamPrimitiveTopology {
    pub fn create(value: t12::EPrimitiveTopologyType) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::PrimitiveTopology.d3dtype(),
            value: value.d3dtype(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamRTVFormats<'a> {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_RT_FORMAT_ARRAY,
    phantom: PhantomData<&'a t12::SRTFormatArray>,
}

impl<'a> SPipelineStateStreamRTVFormats<'a> {
    pub fn create(format_array: &t12::SRTFormatArray) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::RenderTargetFormats.d3dtype(),
            value: format_array.d3dtype(),
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamDepthStencilDesc {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::D3D12_DEPTH_STENCIL_DESC,
}

impl SPipelineStateStreamDepthStencilDesc {
    pub fn create(desc: t12::SDepthStencilDesc) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::DepthStencil.d3dtype(),
            value: desc.d3dtype(),
        }
    }
}

#[repr(C)]
pub struct SPipelineStateStreamDepthStencilFormat {
    type_: win::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: win::DXGI_FORMAT,
}

impl SPipelineStateStreamDepthStencilFormat {
    pub fn create(format: t12::EDXGIFormat) -> Self {
        assert!(size_of::<Self>() % 8 == 0);
        Self {
            type_: t12::EPipelineStateSubobjectType::DepthStencilFormat.d3dtype(),
            value: format.d3dtype(),
        }
    }
}

impl t12::SInputLayoutDesc {
    pub fn create(input_element_descs: &[t12::SInputElementDesc]) -> Self {
        let mut result = Self {
            input_element_descs: ArrayVec::new(),
            d3d_input_element_descs: ArrayVec::new(),
        };

        result
            .input_element_descs
            .try_extend_from_slice(input_element_descs)
            .unwrap();
        result
    }
}
