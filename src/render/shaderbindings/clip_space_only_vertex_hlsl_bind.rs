use niced3d12 as n12;
use typeyd3d12 as t12;
use super::types;

pub struct SClipSpaceOnlyVertexHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SClipSpaceOnlyVertexHLSLBind {
    mvp_rp_idx: usize,
}

impl SClipSpaceOnlyVertexHLSL {
    const LOCALVERTICESSLOT: u32 = 0;

    // -- by convention, spaces 0-2 are for vertex shader use
    const BASESPACE: u32 = 0;

    pub fn new() -> Result<Self, &'static str> {
        let blob = t12::read_file_to_blob("shaders_built/clip_space_only_vertex.cso")?;
        let byte_code = t12::SShaderBytecode::create(blob);

        Ok(Self{
            _bytecode: byte_code,
        })
    }

    pub fn bytecode(&self) -> &t12::SShaderBytecode {
        &self._bytecode
    }

    pub fn input_layout_desc() -> t12::SInputLayoutDesc {
        let input_elements = [types::def_local_verts_input_element(Self::LOCALVERTICESSLOT)];
        t12::SInputLayoutDesc::create(&input_elements)
    }

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc) -> SClipSpaceOnlyVertexHLSLBind {
        let mvp_root_parameter = types::SModelViewProjection::root_parameter(0, Self::BASESPACE as usize);

        root_signature_desc.parameters.push(mvp_root_parameter);
        let mvp_rp_idx = root_signature_desc.parameters.len() - 1;

        SClipSpaceOnlyVertexHLSLBind {
            mvp_rp_idx,
        }
    }

    pub fn set_vertex_buffers(
        &self,
        list: &mut n12::SCommandList,
        local_verts_vbv: &t12::SVertexBufferView,
    )
    {
        list.ia_set_vertex_buffers(0, &[local_verts_vbv]);
    }

    pub fn set_graphics_roots(
        &self,
        bind: &SClipSpaceOnlyVertexHLSLBind,
        list: &mut n12::SCommandList,
        mvp: &types::SModelViewProjection,
    )
    {
        list.set_graphics_root_32_bit_constants(bind.mvp_rp_idx, mvp, 0);
    }
}