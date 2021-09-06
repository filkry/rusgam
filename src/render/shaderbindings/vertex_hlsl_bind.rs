use std::mem::size_of;

use crate::math::Mat4;
use crate::niced3d12 as n12;
use crate::typeyd3d12 as t12;
//use super::types;

pub struct SVertexHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SVertexHLSLBind {
    view_projection_rp_idx: usize,
}

impl SVertexHLSL {
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

    /*
    pub fn input_layout_desc() -> t12::SInputLayoutDesc {
        let input_elements = [];
        t12::SInputLayoutDesc::create(&input_elements)
    }
    */

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc) -> SVertexHLSLBind {
        let vp_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0 as u32,
                    register_space: Self::BASESPACE as u32,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        root_signature_desc.parameters.push(vp_root_parameter);
        let view_projection_rp_idx = root_signature_desc.parameters.len() - 1;

        SVertexHLSLBind {
            view_projection_rp_idx,
        }
    }

    pub fn set_vertex_buffers(
        &self,
        bind: &SVertexHLSLBind,
        list: &mut n12::SCommandList,
        local_verts_descriptor: t12::SGPUVirtualAddress,
        local_normals_descriptor: t12::SGPUVirtualAddress,
        uvs_descriptor: t12::SGPUVirtualAddress,
    )
    {
        list.set_graphics_root_shader_resource_view(bind.vertex_buffer_rp, local_verts_descriptor);
        list.set_graphics_root_shader_resource_view(bind.normals_buffer_rp, local_normals_descriptor);
        list.set_graphics_root_shader_resource_view(bind.uvs_descriptor, uvs_descriptor);
    }

    pub fn set_graphics_roots(
        &self,
        bind: &SVertexHLSLBind,
        list: &mut n12::SCommandList,
        vp: &Mat4,
    )
    {
        list.set_graphics_root_32_bit_constants(bind.view_projection_rp_idx, vp, 0);
    }
}