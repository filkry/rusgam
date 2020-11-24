use entity::{SEntityBucket};
use model::{SMeshLoader};
use n12;
use t12;
use super::shaderbindings;

#[repr(C)]
struct SComputeSkinningPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    compute_shader: n12::SPipelineStateStreamComputeShader<'a>,
}

pub struct SComputeSkinningPipeline {
    compute_shader: shaderbindings::SComputeSkinningHLSL,
    compute_shader_bind: shaderbindings::SComputeSkinningHLSLBind,

    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,
}

pub fn setup_pipeline(
    device: &n12::SDevice,
) -> Result<SComputeSkinningPipeline, &'static str> {
    let compute_shader = shaderbindings::SComputeSkinningHLSL::new()?;

    let root_signature_flags = t12::SRootSignatureFlags::create(&[
        t12::ERootSignatureFlags::DenyVertexShaderRootAccess,
        t12::ERootSignatureFlags::DenyHullShaderRootAccess,
        t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
        t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        t12::ERootSignatureFlags::DenyPixelShaderRootAccess,
    ]);

    let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
    let compute_shader_bind = compute_shader.bind(&mut root_signature_desc);

    let root_signature =
        device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

    let pipeline_state_stream = SComputeSkinningPipelineStateStream {
        root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
        compute_shader: n12::SPipelineStateStreamComputeShader::create(compute_shader.bytecode()),
    };

    let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
    let pipeline_state = device
        .raw()
        .create_pipeline_state(&pipeline_state_stream_desc)?;

    Ok(SComputeSkinningPipeline {
        compute_shader,
        compute_shader_bind,

        root_signature,
        pipeline_state,
    })
}

impl SComputeSkinningPipeline {
    pub fn compute(
        &self,
        command_list: &mut n12::SCommandList,
        mesh_loader: &SMeshLoader,
        entities: &SEntityBucket,
    ) {

        command_list.set_pipeline_state(&self.pipeline_state);
        command_list.set_compute_root_signature(&self.root_signature.raw());

        /*
        for entity in entities.entities() {
            if let Some(skinning) = entity.model_skinning {
                let model = entity.model.expect("skinning without model");

                let local_verts_vbv = mesh_loader.local_verts_vbv(model.mesh);
                let local_normals_vbv = mesh_loader.local_normals_vbv(model.mesh);

                let mesh_skinning = mesh_loader.get_mesh_skinning(model.mesh).expect("model skinning without mesh skinning");

                self.compute_shader.set_compute_roots(
                    &self.compute_shader_bind,
                    command_list,
                    skinning.joints_bind_to_cur_view.gpu_descriptor(0),
                    local_verts_vbv,
                    local_normals_vbv,
                    mesh_skinning.vertex_skinning_buffer_view.gpu_descriptor(0),
                    skinning.skinned_verts_vbv,
                    skinning.skinned_normals_vbv,
                );

                let num_verts = mesh_loader.vertex_count(model.mesh);
                let num_groups = (num_verts / 64) + (if num_verts % 64 != 0 { 1 } else { 0 });
                command_list.dispatch(num_groups, 0, 0);
            }
        }
        */
    }
}
