use entity::{SEntityBucket};
use entity_model;
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
        entities: &mut SEntityBucket,
        entity_model: &entity_model::SBucket,
    ) {

        command_list.set_pipeline_state(&self.pipeline_state);
        command_list.set_compute_root_signature(&self.root_signature.raw());

        let entities_mut = entities.entities_mut();

        assert!(false, "Should loop over models (or even model skinning!");
        for ei in 0..entities_mut.max() {
            if let None = entities_mut.get_by_index_mut(ei).expect("loop bounded by max") {
                continue;
            }

            let entity_handle = entities_mut.handle_for_index(ei).unwrap();
            let entity = entities_mut.get_by_index_mut(ei).expect("loop bounded by max").expect("checked None above");

            if let Some(skinning) = &mut entity.model_skinning {
                skinning.update_skinning_joint_buffer(mesh_loader);

                let model_handle = entity_model.handle_for_entity(entity_handle).unwrap();
                let model = entity_model.get_model(model_handle);

                let local_verts_address = mesh_loader.local_verts_resource(model.mesh).raw.raw().get_gpu_virtual_address();
                let local_normals_address = mesh_loader.local_normals_resource(model.mesh).raw.raw().get_gpu_virtual_address();

                let mesh_skinning = mesh_loader.get_mesh_skinning(model.mesh).expect("model skinning without mesh skinning");

                self.compute_shader.set_compute_roots(
                    &self.compute_shader_bind,
                    command_list,
                    skinning.joints_bind_to_cur_resource.raw.raw().get_gpu_virtual_address(),
                    local_verts_address,
                    local_normals_address,
                    mesh_skinning.vertex_skinning_buffer_resource.raw.raw().get_gpu_virtual_address(),
                    skinning.skinned_verts_resource.raw.raw().get_gpu_virtual_address(),
                    skinning.skinned_normals_resource.raw.raw().get_gpu_virtual_address(),
                );

                let num_verts = mesh_loader.vertex_count(model.mesh);
                let num_groups = (num_verts / 64) + (if num_verts % 64 != 0 { 1 } else { 0 });
                command_list.dispatch(num_groups as u32, 1, 1);

                command_list.transition_resource(
                    &skinning.skinned_verts_resource.raw,
                    t12::EResourceStates::UnorderedAccess,
                    t12::EResourceStates::VertexAndConstantBuffer,
                ).unwrap();
                command_list.transition_resource(
                    &skinning.skinned_normals_resource.raw,
                    t12::EResourceStates::UnorderedAccess,
                    t12::EResourceStates::VertexAndConstantBuffer,
                ).unwrap();
            }
        }
    }
}
