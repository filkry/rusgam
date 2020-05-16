// -- std includes
use std::ops::{Deref};
use std::mem::{size_of};

// -- crate includes
use arrayvec::{ArrayVec};
use glm::{Vec3, Vec4, Mat4};

use niced3d12 as n12;
use typeyd3d12 as t12;
use allocate::{SMemVec, STACK_ALLOCATOR, SYSTEM_ALLOCATOR};
use model;
use model::{SModel};
use utils::{STransform};

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SMeshPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SLinePipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[allow(dead_code)]
struct SLine {
    start: Vec3,
    end: Vec3,
    colour: Vec3,
}

pub struct SRenderTemp<'a> {
    line_pipeline_state: t12::SPipelineState,
    line_root_signature: n12::SRootSignature,
    line_vp_root_param_idx: usize,
    _line_vert_byte_code: t12::SShaderBytecode,
    _line_pixel_byte_code: t12::SShaderBytecode,

    lines: SMemVec::<'a, SLine>,
    line_vertex_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_view: [Option<t12::SVertexBufferView>; 2],
    line_in_world_indices: SMemVec::<'a, u16>,
    line_over_world_indices: SMemVec::<'a, u16>,

    mesh_pipeline_state: t12::SPipelineState,
    mesh_root_signature: n12::SRootSignature,
    mesh_mvp_root_param_idx: usize,
    mesh_color_root_param_idx: usize,
    _mesh_vert_byte_code: t12::SShaderBytecode,
    _mesh_pixel_byte_code: t12::SShaderBytecode,

    models: SMemVec::<'a, SModel>,
    model_xforms: SMemVec::<'a, STransform>,
    model_in_world_indices: SMemVec::<'a, u16>,
    model_over_world_indices: SMemVec::<'a, u16>,
}

impl<'a> SRenderTemp<'a> {

    pub fn new(device: &n12::SDevice) -> Result<Self, &'static str> {
        // =========================================================================================
        // LINE pipeline state
        // =========================================================================================

        let line_root_signature_flags = {
            use t12::ERootSignatureFlags::*;

            t12::SRootSignatureFlags::create(&[
                AllowInputAssemblerInputLayout,
                DenyHullShaderRootAccess,
                DenyDomainShaderRootAccess,
                DenyGeometryShaderRootAccess,
                DenyPixelShaderRootAccess,
            ])
        };

        let vp_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let mut line_root_signature_desc = t12::SRootSignatureDesc::new(line_root_signature_flags);
        line_root_signature_desc.parameters.push(vp_root_parameter);
        let line_vp_root_param_idx = line_root_signature_desc.parameters.len() - 1;

        let line_root_signature =
            device.create_root_signature(line_root_signature_desc,
                                         t12::ERootSignatureVersion::V1)?;

        let mut line_input_layout_desc = t12::SInputLayoutDesc::create(&[
            t12::SInputElementDesc::create(
                "POSITION",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "COLOR",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ]);

        let line_vertblob = t12::read_file_to_blob("shaders_built/debug_line_vertex.cso")?;
        let line_pixelblob = t12::read_file_to_blob("shaders_built/debug_line_pixel.cso")?;

        let line_vert_byte_code = t12::SShaderBytecode::create(line_vertblob);
        let line_pixel_byte_code = t12::SShaderBytecode::create(line_pixelblob);

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        let line_pipeline_state_stream = SLinePipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&line_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut line_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Line,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&line_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&line_pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let line_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&line_pipeline_state_stream);
        let line_pipeline_state = device
            .raw()
            .create_pipeline_state(&line_pipeline_state_stream_desc)?;

        // =========================================================================================
        // MESH/MODEL pipeline state
        // =========================================================================================
        let mesh_root_signature_flags = {
            use t12::ERootSignatureFlags::*;

            t12::SRootSignatureFlags::create(&[
                AllowInputAssemblerInputLayout,
                DenyHullShaderRootAccess,
                DenyDomainShaderRootAccess,
                DenyGeometryShaderRootAccess,
            ])
        };

        let mvp_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };
        let color_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 1,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Vec4>() / size_of::<f32>()) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        let mut mesh_root_signature_desc = t12::SRootSignatureDesc::new(mesh_root_signature_flags);
        mesh_root_signature_desc.parameters.push(mvp_root_parameter);
        let mesh_mvp_root_param_idx = mesh_root_signature_desc.parameters.len() - 1;
        mesh_root_signature_desc.parameters.push(color_root_parameter);
        let mesh_color_root_param_idx = mesh_root_signature_desc.parameters.len() - 1;
        let mesh_root_signature = device.create_root_signature(
            mesh_root_signature_desc, t12::ERootSignatureVersion::V1)?;

        let mut mesh_input_layout_desc = model::model_per_vertex_input_layout_desc();

        let mesh_vertblob = t12::read_file_to_blob("shaders_built/temp_mesh_vertex.cso")?;
        let mesh_pixelblob = t12::read_file_to_blob("shaders_built/temp_mesh_pixel.cso")?;
        let mesh_vert_byte_code = t12::SShaderBytecode::create(mesh_vertblob);
        let mesh_pixel_byte_code = t12::SShaderBytecode::create(mesh_pixelblob);

        let mesh_pipeline_state_stream = SMeshPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&mesh_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut mesh_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&mesh_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&mesh_pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let mesh_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&mesh_pipeline_state_stream);
        let mesh_pipeline_state = device
            .raw()
            .create_pipeline_state(&mesh_pipeline_state_stream_desc)?;

        Ok(Self{
            line_pipeline_state,
            line_root_signature,
            line_vp_root_param_idx,
            _line_vert_byte_code: line_vert_byte_code,
            _line_pixel_byte_code: line_pixel_byte_code,
            lines: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            line_vertex_buffer_intermediate_resource: [None, None],
            line_vertex_buffer_resource: [None, None],
            line_vertex_buffer_view: [None, None],
            line_in_world_indices: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            line_over_world_indices: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,

            mesh_pipeline_state,
            mesh_root_signature,
            mesh_mvp_root_param_idx,
            mesh_color_root_param_idx,
            _mesh_vert_byte_code: mesh_vert_byte_code,
            _mesh_pixel_byte_code: mesh_pixel_byte_code,

            models: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            model_xforms: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            model_in_world_indices: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            model_over_world_indices: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
        })
    }

    pub fn draw_model(&mut self, model: &SModel, location: &STransform, over_world: bool) {
        assert!(model.diffuse_texture.is_none());

        self.models.push(model.clone());
        self.model_xforms.push(location.clone());
        assert!(self.models.len() == self.model_xforms.len());
        let idx = (self.models.len() - 1) as u16;
        if over_world {
            self.model_over_world_indices.push(idx);
        }
        else {
            self.model_in_world_indices.push(idx);
        }
    }

    pub fn draw_line(&mut self, start: &Vec3, end: &Vec3, color: &Vec3, over_world: bool) {
        self.lines.push(SLine {
            start: start.clone(),
            end: end.clone(),
            colour: color.clone(),
        });
        let idx = (self.lines.len() - 1) as u16;
        if over_world {
            self.line_over_world_indices.push(idx);
        }
        else {
            self.line_in_world_indices.push(idx);
        }
    }
}

impl<'a> super::SRender<'a> {
    pub fn render_temp_in_world(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4) -> Result<(), &'static str> {
        self.render_temp_lines(window, view_matrix, true)?;
        self.render_temp_models(window, view_matrix, true)?;

        Ok(())
    }

    pub fn render_temp_over_world(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4) -> Result<(), &'static str> {
        self.render_temp_lines(window, view_matrix, false)?;
        self.render_temp_models(window, view_matrix, false)?;

        Ok(())
    }

    pub fn render_temp_lines(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4, in_world: bool) -> Result<(), &'static str> {
        let back_buffer_idx = window.currentbackbufferindex();

        /* A very basic test
        tr.lines.push(SDebugLine{
            start: Vec3::new(-5.0, 2.0, 0.0),
            end: Vec3::new(5.0, 2.0, 0.0),
            colour: Vec3::new(1.0, 0.0, 0.0),
        });
        */

        if self.render_temp.lines.len() == 0 {
            return Ok(());
        }

        // -- create/upload vertex buffer
        // -- must match SDebugLineShaderVert in debug_line_vertex.hlsl
        #[repr(C)]
        struct SDebugLineShaderVert {
            pos: [f32; 3],
            colour: [f32; 3],
        }
        impl SDebugLineShaderVert {
            fn new(pos: &Vec3, colour: &Vec3) -> Self {
                Self {
                    pos: [pos.x, pos.y, pos.z],
                    colour: [colour.x, colour.y, colour.z],
                }
            }
        }

        // -- generate data and copy to GPU
        // -- $$$FRK(TODO): move this step to earlier in render?
        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let tr = &mut self.render_temp;

            let mut vertex_buffer_data = SMemVec::new(
                sa,
                tr.lines.len() * 2,
                0,
            )?;

            let line_indices = if in_world { &tr.line_in_world_indices } else { &tr.line_over_world_indices };

            for i in line_indices.as_slice() {
                let line = &tr.lines[*i as usize];
                vertex_buffer_data.push(SDebugLineShaderVert::new(&line.start, &line.colour));
                vertex_buffer_data.push(SDebugLineShaderVert::new(&line.end, &line.colour));
            }

            let handle = self.copy_command_pool.alloc_list()?;
            let mut copy_command_list = self.copy_command_pool.get_list(handle)?;

            let vert_buffer_resource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copy_command_list.update_buffer_resource(
                    self.device.deref(),
                    vertex_buffer_data.as_slice(),
                    vertbufferflags
                )?
            };
            let vertex_buffer_view = vert_buffer_resource
                .destinationresource
                .create_vertex_buffer_view()?;

            drop(copy_command_list);
            let fence_val = self.copy_command_pool.execute_and_free_list(handle)?;
            drop(handle);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_pool.gpu_wait(
                self.copy_command_pool.get_internal_fence(),
                fence_val,
            )?;

            tr.line_vertex_buffer_intermediate_resource[back_buffer_idx] =
                Some(vert_buffer_resource.intermediateresource);
            tr.line_vertex_buffer_resource[back_buffer_idx] =
                Some(vert_buffer_resource.destinationresource);
            tr.line_vertex_buffer_view[back_buffer_idx] = Some(vertex_buffer_view);

            Ok(())
        })?;

        // -- set up pipeline and render lines
        let handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(handle)?;

        list.set_pipeline_state(&self.render_temp.line_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.line_root_signature.raw());

        // -- setup rasterizer state
        let viewport = t12::SViewport::new(
            0.0,
            0.0,
            window.width() as f32,
            window.height() as f32,
            None,
            None,
        );
        list.rs_set_viewports(&[&viewport]);

        // -- setup the output merger
        let render_target_view = window.currentrendertargetdescriptor()?;
        let depth_texture_view = self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

        let perspective_matrix: Mat4 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh_zo(aspect, self.fovy(), self.znear(), zfar)
        };
        let view_perspective = perspective_matrix * view_matrix;

        list.set_graphics_root_32_bit_constants(self.render_temp.line_vp_root_param_idx as u32,
                                                &view_perspective, 0);

        // -- set up input assembler
        list.ia_set_primitive_topology(t12::EPrimitiveTopology::LineList);
        let vert_buffer_view = self.render_temp.line_vertex_buffer_view[back_buffer_idx].
            as_ref().expect("should have generated resource earlier in this function");
        list.ia_set_vertex_buffers(0, &[vert_buffer_view]);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        for i in 0..self.render_temp.lines.len() {
            list.draw_instanced(2, 1, (i * 2) as u32, 0);
        }

        // -- execute on the queue
        drop(list);
        assert_eq!(window.currentbackbufferindex(), back_buffer_idx);
        self.direct_command_pool.execute_and_free_list(handle)?;

        self.render_temp.lines.clear();

        Ok(())
    }

    pub fn render_temp_models(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4, in_world: bool) -> Result<(), &'static str> {
        if self.render_temp.models.len() == 0 {
            return Ok(());
        }
        assert!(self.render_temp.models.len() == self.render_temp.model_xforms.len());

        let back_buffer_idx = window.currentbackbufferindex();

        // -- set up pipeline and render models
        let handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(handle)?;

        list.set_pipeline_state(&self.render_temp.mesh_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.mesh_root_signature.raw());

        // -- setup rasterizer state
        let viewport = t12::SViewport::new(
            0.0,
            0.0,
            window.width() as f32,
            window.height() as f32,
            None,
            None,
        );
        list.rs_set_viewports(&[&viewport]);

        // -- setup the output merger
        let render_target_view = window.currentrendertargetdescriptor()?;
        let depth_texture_view = self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

        let perspective_matrix: Mat4 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh_zo(aspect, self.fovy(), self.znear(), zfar)
        };
        let view_projection = perspective_matrix * view_matrix;

        list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        let indices = if in_world { &self.render_temp.model_in_world_indices } else { &self.render_temp.model_over_world_indices };

        for i in indices.as_slice() {
            let ii = *i as usize;
            let model_matrix = self.render_temp.model_xforms[ii].as_mat4();
            let mvp = view_projection * model_matrix;

            list.set_graphics_root_32_bit_constants(self.render_temp.mesh_mvp_root_param_idx as u32,
                                                    &mvp, 0);
            list.set_graphics_root_32_bit_constants(self.render_temp.mesh_color_root_param_idx as u32,
                                                    &self.render_temp.models[ii].diffuse_colour, 0);

            self.mesh_loader.bind_buffers_and_draw(self.render_temp.models[ii].mesh, &mut list)?;
        }

        // -- execute on the queue
        drop(list);
        assert_eq!(window.currentbackbufferindex(), back_buffer_idx);
        self.direct_command_pool.execute_and_free_list(handle)?;

        self.render_temp.models.clear();
        self.render_temp.model_xforms.clear();

        Ok(())
    }
}