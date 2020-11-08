use std::mem::{size_of};
use std::cell::{RefMut};

use niced3d12 as n12;
use typeyd3d12 as t12;
use glm::{Mat4};
use utils::{STransform};
use super::types;

// -- used to fill out shader metadata, must match SModelViewProjection in vertex.hlsl
#[repr(C)]
#[derive(Debug)]
pub struct SModelViewProjection {
    model: Mat4,
    view_projection: Mat4,
    mvp: Mat4,
}

impl SModelViewProjection {
    pub fn new(view_projection: &Mat4, model_xform: &STransform) -> Self {
        let model_matrix = model_xform.as_mat4();

        let mvp_matrix = view_projection * model_matrix;
        Self{
            model: model_matrix.clone(),
            view_projection: view_projection.clone(),
            mvp: mvp_matrix,
        }
    }
}

pub struct SVertexHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SVertexHLSLBind {
    mvp_rp_idx: usize,
}

impl SVertexHLSL {
    const BASEVERTEXDATASLOT: usize = 0;

    // -- by convention, spaces 0-2 are for vertex shader use
    const BASESPACE: u32 = 0;

    pub fn new() -> Result<Self, &'static str> {
        let blob = t12::read_file_to_blob("shaders_built/vertex.cso")?;
        let byte_code = t12::SShaderBytecode::create(blob);

        Ok(Self{
            _bytecode: byte_code,
        })
    }

    pub fn bytecode(&self) -> &t12::SShaderBytecode {
        &self._bytecode
    }

    pub fn input_layout_desc() -> t12::SInputLayoutDesc {
        let input_elements = types::SBaseVertexData::new_input_elements(Self::BASEVERTEXDATASLOT);
        t12::SInputLayoutDesc::create(&input_elements)
    }

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc) -> SVertexHLSLBind {
        let mvp_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 0,
                    register_space: Self::BASESPACE,
                    num_32_bit_values: (size_of::<SModelViewProjection>() / size_of::<f32>()) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        root_signature_desc.parameters.push(mvp_root_parameter);
        let mvp_rp_idx = root_signature_desc.parameters.len() - 1;

        SVertexHLSLBind {
            mvp_rp_idx,
        }
    }

    pub fn set_graphics_roots(
        &self,
        bind: &SVertexHLSLBind,
        list: &mut RefMut<n12::SCommandList>,
        mvp: &SModelViewProjection,
    )
    {
        list.set_graphics_root_32_bit_constants(bind.mvp_rp_idx, mvp, 0);
    }
}