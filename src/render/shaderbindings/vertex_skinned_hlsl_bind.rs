#![allow(dead_code)]

use niced3d12 as n12;
use typeyd3d12 as t12;
use super::types;

pub struct SVertexSkinnedHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SVertexSkinnedHLSLBind {
    mvp_rp_idx: usize,
    jointworldtransforms_rp_idx: usize,
}

impl SVertexSkinnedHLSL {
    const BASEVERTEXDATASLOT: usize = 0;
    const SKINNINGDATASLOT: usize = 1;

    // -- by convention, spaces 0-2 are for vertex shader use
    const BASESPACE: u32 = 0;

    pub fn new() -> Result<Self, &'static str> {
        let blob = t12::read_file_to_blob("shaders_built/vertex_skinned.cso")?;
        let byte_code = t12::SShaderBytecode::create(blob);

        Ok(Self{
            _bytecode: byte_code,
        })
    }

    pub fn bytecode(&self) -> &t12::SShaderBytecode {
        &self._bytecode
    }

    pub fn input_layout_desc() -> t12::SInputLayoutDesc {
        let base_vert_elements = types::SBaseVertexData::new_input_elements(Self::BASEVERTEXDATASLOT);

        let skinning_elements = [
            t12::SInputElementDesc::create(
                "JOINTS",
                0,
                t12::EDXGIFormat::R32UINT,
                Self::SKINNINGDATASLOT as u32,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "JOINTWEIGHTS",
                0,
                t12::EDXGIFormat::R32G32B32A32Float,
                Self::SKINNINGDATASLOT as u32,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ];

        let combined = [
            base_vert_elements[0],
            base_vert_elements[1],
            base_vert_elements[2],
            skinning_elements[0],
            skinning_elements[1],
        ];

        t12::SInputLayoutDesc::create(&combined)
    }

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc) -> SVertexSkinnedHLSLBind {
        let mvp_root_parameter = types::SModelViewProjection::root_parameter(0, Self::BASESPACE as usize);

        root_signature_desc.parameters.push(mvp_root_parameter);
        let mvp_rp_idx = root_signature_desc.parameters.len() - 1;

        let joints_root_paramater = t12::SRootParameter {
            type_: t12::ERootParameterType::SRV,
            type_data: t12::ERootParameterTypeData::Descriptor {
                descriptor: t12::SRootDescriptor {
                    shader_register: 0,
                    register_space: Self::BASESPACE as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };
        root_signature_desc.parameters.push(joints_root_paramater);
        let jointworldtransforms_rp_idx = root_signature_desc.parameters.len() - 1;

        SVertexSkinnedHLSLBind {
            mvp_rp_idx,
            jointworldtransforms_rp_idx,
        }
    }

    pub fn set_graphics_roots(
        &self,
        bind: &SVertexSkinnedHLSLBind,
        list: &mut n12::SCommandList,
        mvp: &types::SModelViewProjection,
        jointworldtransforms_descriptor: t12::SGPUDescriptorHandle,
    )
    {
        list.set_graphics_root_32_bit_constants(bind.mvp_rp_idx, mvp, 0);
        list.set_graphics_root_shader_resource_view(bind.jointworldtransforms_rp_idx, jointworldtransforms_descriptor);
    }
}