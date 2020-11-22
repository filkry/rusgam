use std::mem::{size_of};

use glm::{Mat4};
use typeyd3d12 as t12;
use utils::{STransform};

// -- must match SVertexSkinningData in vertex_skinned.hlsl
#[repr(C)]
pub struct SVertexSkinningData {
    joints: [u32; 4],
    joint_weights: [f32; 4],
}

// -- used to fill out shader metadata, must match SModelViewProjection in types.hlsl
#[repr(C)]
#[derive(Debug)]
pub struct SModelViewProjection {
    model: Mat4,
    view_projection: Mat4,
    mvp: Mat4,
}

pub fn def_local_verts_input_element(slot: u32) -> t12::SInputElementDesc {
    t12::SInputElementDesc::create(
        "POSITION",
        0,
        t12::EDXGIFormat::R32G32B32Float,
        slot as u32,
        winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
        t12::EInputClassification::PerVertexData,
        0,
    )
}

pub fn def_local_normals_input_element(slot: u32) -> t12::SInputElementDesc {
    t12::SInputElementDesc::create(
        "NORMAL",
        0,
        t12::EDXGIFormat::R32G32B32Float,
        slot as u32,
        winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
        t12::EInputClassification::PerVertexData,
        0,
    )
}

pub fn def_uvs_input_element(slot: u32) -> t12::SInputElementDesc {
    t12::SInputElementDesc::create(
        "TEXCOORD",
        0,
        t12::EDXGIFormat::R32G32Float,
        slot as u32,
        winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
        t12::EInputClassification::PerVertexData,
        0,
    )
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

    pub fn root_parameter(shader_register: usize, register_space: usize) -> t12::SRootParameter {
        t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: shader_register as u32,
                    register_space: register_space as u32,
                    num_32_bit_values: (size_of::<SModelViewProjection>() / size_of::<f32>()) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        }
    }
}


