//use std::ops::{Deref};
use std::rc::{Weak};

use model;
use n12;
use t12;
use utils;
use utils::{STransform};

use glm::{Vec3, Mat4};

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SShadowPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
}

pub struct SShadowMappingPipeline {
    _vertex_byte_code: t12::SShaderBytecode,
    _pixel_byte_code: t12::SShaderBytecode,

    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,

    shadow_cube_width: usize,
    shadow_cube_height: usize,

    shadow_depth_resource: n12::SResource,
    shadow_depth_view: n12::SDescriptorAllocatorAllocation,
    shadow_srv: n12::SDescriptorAllocatorAllocation,
}

pub fn setup_shadow_mapping_pipeline(
    device: &n12::SDevice,
    direct_command_pool: &mut n12::SCommandListPool,
    dsv_heap: Weak<n12::SDescriptorAllocator>,
    srv_heap: Weak<n12::SDescriptorAllocator>,
    shadow_cube_width: usize,
    shadow_cube_height: usize,
) -> Result<SShadowMappingPipeline, &'static str> {
    let vertex_blob = t12::read_file_to_blob("shaders_built/shadow_vertex.cso")?;
    let pixel_blob = t12::read_file_to_blob("shaders_built/shadow_pixel.cso")?;

    let vertex_byte_code = t12::SShaderBytecode::create(vertex_blob);
    let pixel_byte_code = t12::SShaderBytecode::create(pixel_blob);

    let mut input_layout_desc = model::model_per_vertex_input_layout_desc();

    let mvp_root_parameter = t12::SRootParameter {
        type_: t12::ERootParameterType::E32BitConstants,
        type_data: t12::ERootParameterTypeData::Constants {
            constants: t12::SRootConstants {
                shader_register: 0,
                register_space: 0,
                num_32_bit_values: (std::mem::size_of::<Mat4>() * 3 / 4) as u32,
            },
        },
        shader_visibility: t12::EShaderVisibility::Vertex,
    };

    let root_signature_flags = t12::SRootSignatureFlags::create(&[
        t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
        t12::ERootSignatureFlags::DenyHullShaderRootAccess,
        t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
        t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        t12::ERootSignatureFlags::DenyPixelShaderRootAccess,
    ]);

    let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
    root_signature_desc.parameters.push(mvp_root_parameter);

    let root_signature =
        device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

    let pipeline_state_stream = SShadowPipelineStateStream {
        root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
        input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
        primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
            t12::EPrimitiveTopologyType::Triangle,
        ),
        vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vertex_byte_code),
        pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
        depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
            t12::EDXGIFormat::D32Float,
        ),
    };

    let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
    let pipeline_state = device
        .raw()
        .create_pipeline_state(&pipeline_state_stream_desc)?;

    // -- depth texture
    #[allow(unused_variables)]
    let (resource, view) = n12::create_committed_depth_textures(
        shadow_cube_width as u32,
        shadow_cube_height as u32,
        6,
        &device,
        t12::EDXGIFormat::R32Typeless,
        t12::EResourceStates::GenericRead,
        direct_command_pool,
        &dsv_heap.upgrade().expect("dsv freed"),
    )?;

    let srv = {
        let descriptors = n12::descriptorallocator::descriptor_alloc(&srv_heap.upgrade().expect("heap freed"), 1)?;

        device.create_shader_resource_view(
            &resource,
            &t12::SShaderResourceViewDesc {
                format: t12::EDXGIFormat::R32Float,
                view: t12::ESRV::TextureCube(t12::STexCubeSRV::default()),
            },
            descriptors.cpu_descriptor(0),
        )?;

        descriptors
    };

    Ok(SShadowMappingPipeline {
        _vertex_byte_code: vertex_byte_code,
        _pixel_byte_code: pixel_byte_code,

        root_signature,
        pipeline_state,

        shadow_cube_width,
        shadow_cube_height,

        shadow_depth_resource: resource,
        shadow_depth_view: view,
        shadow_srv: srv,
    })
}

impl SShadowMappingPipeline {
    pub fn render(
        &self,
        mesh_loader: &model::SMeshLoader,
        light_pos_world: &Vec3,
        cl: &mut n12::SCommandList,
        models: &[model::SModel],
        model_matrices: &[STransform],
    ) -> Result<(), &'static str> {

        // -- all this data could be cached ----------------------------------------
        let perspective_matrix: Mat4 = {
            let aspect = 1.0;
            let fovy: f32 = utils::PI / 2.0;
            let znear = 0.1;
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh_zo(aspect, fovy, znear, zfar)
        };

        //println!("{:?}", perspective_matrix);

        let viewport = t12::SViewport::new(
            0.0,
            0.0,
            self.shadow_cube_width as f32,
            self.shadow_cube_height as f32,
            None,
            None,
        );
        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };

        let dirs = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, -1.0),
        ];
        // -- all this data could be cached ----------------------------------------

        cl.transition_resource(
            &self.shadow_depth_resource,
            t12::EResourceStates::GenericRead,
            t12::EResourceStates::DepthWrite,
        )?;

        cl.set_pipeline_state(&self.pipeline_state);
        cl.set_graphics_root_signature(&self.root_signature.raw());

        cl.rs_set_viewports(&[&viewport]);
        cl.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));


        for (i, dir) in dirs.iter().enumerate() {
            cl.clear_depth_stencil_view(self.shadow_depth_view.cpu_descriptor(i), 1.0)?;
            cl.om_set_render_targets(&[], false, &self.shadow_depth_view.cpu_descriptor(i));

            let up = {
                if dir.y == 1.0 {
                    Vec3::new(0.0, 0.0, -1.0)
                }
                else if dir.y == -1.0 {
                    Vec3::new(0.0, 0.0, 1.0)
                }
                else {
                    Vec3::new(0.0, 1.0, 0.0)
                }
            };

            let view_matrix = glm::look_at_lh(&light_pos_world, &(light_pos_world + dir), &up);

            let view_perspective = perspective_matrix * view_matrix;

            for modeli in 0..models.len() {
                mesh_loader.render(models[modeli].mesh, cl, &view_perspective, &model_matrices[modeli])?;
            }
        }

        cl.transition_resource(
            &self.shadow_depth_resource,
            t12::EResourceStates::DepthWrite,
            t12::EResourceStates::GenericRead,
        )?;

        Ok(())
    }

    pub fn srv(&self) -> &n12::SDescriptorAllocatorAllocation {
        &self.shadow_srv
    }
}