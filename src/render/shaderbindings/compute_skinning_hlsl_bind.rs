#![allow(dead_code)]

use niced3d12 as n12;
use typeyd3d12 as t12;

pub struct SComputeSkinningHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SComputeSkinningHLSLBind {
    joint_bind_to_cur_rp_idx: usize,
    local_verts_rp_idx: usize,
    local_normals_rp_idx: usize,
    vertex_skinning_rp_idx: usize,

    skinned_verts_rp_idx: usize,
    skinned_normals_rp_idx: usize,
}

impl SComputeSkinningHLSL {
    const BASESPACE: u32 = 0;

    // -- t registers
    const JOINTWORLDTRANSFORMSREGISTER: u32 = 0;
    const LOCALVERTSREGISTER: u32 = 1;
    const LOCALNORMALSREGISTER: u32 = 2;
    const VERTEXSKINNINGREGISTER: u32 = 3;

    // -- u registers
    const SKINNEDVERTSREGISTER: u32 = 0;
    const SKINNEDNORMALSREGISTER: u32 = 1;

    pub fn new() -> Result<Self, &'static str> {
        let blob = t12::read_file_to_blob("shaders_built/compute_skinning.cso")?;
        let byte_code = t12::SShaderBytecode::create(blob);

        Ok(Self{
            _bytecode: byte_code,
        })
    }

    pub fn bytecode(&self) -> &t12::SShaderBytecode {
        &self._bytecode
    }

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc) -> SComputeSkinningHLSLBind {
        let mut add_param = |param: n12::SRootParameter| -> usize {
            root_signature_desc.parameters.push(param.into_raw());
            root_signature_desc.parameters.len() - 1
        };

        let joint_bind_to_cur_rp_idx = add_param(n12::SRootParameter::new_srv_descriptor(
            Self::JOINTWORLDTRANSFORMSREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));
        let local_verts_rp_idx = add_param(n12::SRootParameter::new_srv_descriptor(
            Self::LOCALVERTSREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));
        let local_normals_rp_idx = add_param(n12::SRootParameter::new_srv_descriptor(
            Self::LOCALNORMALSREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));
        let vertex_skinning_rp_idx = add_param(n12::SRootParameter::new_srv_descriptor(
            Self::VERTEXSKINNINGREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));

        let skinned_verts_rp_idx = add_param(n12::SRootParameter::new_uav_descriptor(
            Self::SKINNEDVERTSREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));
        let skinned_normals_rp_idx = add_param(n12::SRootParameter::new_uav_descriptor(
            Self::SKINNEDNORMALSREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::All,
        ));

        SComputeSkinningHLSLBind {
            joint_bind_to_cur_rp_idx,
            local_verts_rp_idx,
            local_normals_rp_idx,
            vertex_skinning_rp_idx,

            skinned_verts_rp_idx,
            skinned_normals_rp_idx,
        }
    }

    pub fn set_compute_roots(
        &self,
        bind: &SComputeSkinningHLSLBind,
        list: &mut n12::SCommandList,
        joint_bind_to_cur_descriptor: t12::SGPUVirtualAddress,
        local_verts_descriptor: t12::SGPUVirtualAddress,
        local_normals_descriptor: t12::SGPUVirtualAddress,
        vertex_skinning_descriptor: t12::SGPUVirtualAddress,
        skinned_verts_descriptor: t12::SGPUVirtualAddress,
        skinned_normals_descriptor: t12::SGPUVirtualAddress,
    )
    {
        list.set_compute_root_shader_resource_view(bind.joint_bind_to_cur_rp_idx, joint_bind_to_cur_descriptor);
        list.set_compute_root_shader_resource_view(bind.local_verts_rp_idx, local_verts_descriptor);
        list.set_compute_root_shader_resource_view(bind.local_normals_rp_idx, local_normals_descriptor);
        list.set_compute_root_shader_resource_view(bind.vertex_skinning_rp_idx, vertex_skinning_descriptor);

        list.set_compute_root_unordered_access_view(bind.skinned_verts_rp_idx, skinned_verts_descriptor);
        list.set_compute_root_unordered_access_view(bind.skinned_normals_rp_idx, skinned_normals_descriptor);
    }
}