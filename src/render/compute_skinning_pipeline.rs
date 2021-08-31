use entity_animation;
use entity_model;
use model::{SMeshLoader};
use n12;
use t12;
use super::shaderbindings;

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

    let pipeline_state_desc = t12::SComputePipelineStateDesc {
        root_signature: root_signature.raw().clone(),
        compute_shader: compute_shader.bytecode(),
    };

    let pipeline_state = device
        .raw()
        .create_compute_pipeline_state(&pipeline_state_desc)?;

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
        e_animation: &mut entity_animation::SBucket,
        e_model: &entity_model::SBucket,
    ) {

        command_list.set_pipeline_state(&self.pipeline_state);
        command_list.set_compute_root_signature(&self.root_signature.raw());

        for e_anim_instance in e_animation.instances.as_mut() {
            let entity_handle = e_anim_instance.owner;

            e_anim_instance.skinning.update_skinning_joint_buffer(mesh_loader);

            let model_handle = e_model.handle_for_entity(entity_handle).unwrap();
            let model = e_model.get_model(model_handle);

            let local_verts_address = mesh_loader.local_verts_resource(model.mesh).raw.raw().get_gpu_virtual_address();
            let local_normals_address = mesh_loader.local_normals_resource(model.mesh).raw.raw().get_gpu_virtual_address();

            let mesh_skinning = mesh_loader.get_mesh_skinning(model.mesh).expect("model skinning without mesh skinning");

            self.compute_shader.set_compute_roots(
                &self.compute_shader_bind,
                command_list,
                e_anim_instance.skinning.joints_bind_to_cur_resource.raw.raw().get_gpu_virtual_address(),
                local_verts_address,
                local_normals_address,
                mesh_skinning.vertex_skinning_buffer_resource.raw.raw().get_gpu_virtual_address(),
                e_anim_instance.skinning.skinned_verts_resource.raw.raw().get_gpu_virtual_address(),
                e_anim_instance.skinning.skinned_normals_resource.raw.raw().get_gpu_virtual_address(),
            );

            let num_verts = mesh_loader.vertex_count(model.mesh);
            let num_groups = (num_verts / 64) + (if num_verts % 64 != 0 { 1 } else { 0 });
            command_list.dispatch(num_groups as u32, 1, 1);

            command_list.transition_resource(
                &e_anim_instance.skinning.skinned_verts_resource.raw,
                t12::EResourceStates::UnorderedAccess,
                t12::EResourceStates::VertexAndConstantBuffer,
            ).unwrap();
            command_list.transition_resource(
                &e_anim_instance.skinning.skinned_normals_resource.raw,
                t12::EResourceStates::UnorderedAccess,
                t12::EResourceStates::VertexAndConstantBuffer,
            ).unwrap();
        }
    }
}
