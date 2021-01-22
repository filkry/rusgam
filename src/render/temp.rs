// -- std includes
use std::ops::{Deref};
use std::mem::{size_of};

// -- crate includes
use arrayvec::{ArrayVec};
use crate::math::{Vec3, Vec4, Mat4};

use crate::niced3d12 as n12;
use crate::typeyd3d12 as t12;
use crate::allocate::{STACK_ALLOCATOR, SYSTEM_ALLOCATOR};
use crate::collections::{SVec};
use crate::model::{SModel, SMeshLoader, STextureLoader, SMeshHandle};
use super::shaderbindings;
use crate::utils::{STransform, SAABB};
use super::{SRenderContext};

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
struct SPointPipelineStateStream<'a> {
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

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SSpherePipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[derive(PartialEq, Clone, Copy)]
pub struct SToken {
    token: u64,
}

#[allow(dead_code)]
struct SPoint {
    p: Vec3,
    colour: Vec3,
    over_world: bool,
    token: SToken,
}

#[allow(dead_code)]
struct SLine {
    start: Vec3,
    end: Vec3,
    colour: Vec4,
    over_world: bool,
    token: SToken,
}

#[allow(dead_code)]
struct SSphere {
    scale: f32,
    pos: Vec3,
    colour: Vec4,
    over_world: bool,
    token: SToken,
}

struct STempModel {
    model: SModel,
    location: STransform,
    over_world: bool,
    token: SToken,
}

pub struct SRenderTemp {
    // -- point pipelines stuff
    point_pipeline_state: t12::SPipelineState,
    point_root_signature: n12::SRootSignature,
    point_vp_root_param_idx: usize,
    _point_vert_byte_code: t12::SShaderBytecode,
    _point_pixel_byte_code: t12::SShaderBytecode,

    points: SVec::<SPoint>,
    point_vertex_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    point_vertex_buffer_resource: [Option<n12::SResource>; 2],
    point_vertex_buffer_view: [Option<t12::SVertexBufferView>; 2],

    // -- line pipeline stuff
    line_pipeline_state: t12::SPipelineState,
    line_root_signature: n12::SRootSignature,
    line_vp_root_param_idx: usize,
    _line_vert_byte_code: t12::SShaderBytecode,
    _line_pixel_byte_code: t12::SShaderBytecode,

    lines: SVec::<SLine>,
    line_vertex_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_view: [Option<t12::SVertexBufferView>; 2],

    // -- sphere pipeline stuff
    instance_mesh_pipeline_state: t12::SPipelineState,
    instance_mesh_root_signature: n12::SRootSignature,
    instance_mesh_vp_root_param_idx: usize,
    _instance_mesh_vert_byte_code: t12::SShaderBytecode,
    _instance_mesh_pixel_byte_code: t12::SShaderBytecode,

    spheres: SVec::<SSphere>,
    sphere_mesh: SMeshHandle,
    sphere_instance_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    sphere_instance_buffer_resource: [Option<n12::SResource>; 2],
    sphere_instance_buffer_view: [Option<t12::SVertexBufferView>; 2],

    // -- mesh pipeline stuff
    mesh_pipeline_state: t12::SPipelineState,
    mesh_root_signature: n12::SRootSignature,
    mesh_mvp_root_param_idx: usize,
    mesh_color_root_param_idx: usize,
    _mesh_vert_byte_code: t12::SShaderBytecode,
    _mesh_pixel_byte_code: t12::SShaderBytecode,

    models: SVec::<STempModel>,

    next_token: u64,
}

impl SToken {
    fn new(token: u64) -> Self {
        Self { token }
    }
}

impl Default for SToken {
    fn default() -> Self {
        Self::new(std::u64::MAX)
    }
}

impl SRenderTemp {

    pub fn new(device: &n12::SDevice, mesh_loader: &mut SMeshLoader, texture_loader: &mut STextureLoader) -> Result<Self, &'static str> {
        // =========================================================================================
        // POINT pipeline state
        // =========================================================================================

        let point_root_signature_flags = {
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
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let mut point_root_signature_desc = t12::SRootSignatureDesc::new(point_root_signature_flags);
        point_root_signature_desc.parameters.push(vp_root_parameter);
        let point_vp_root_param_idx = point_root_signature_desc.parameters.len() - 1;

        let point_root_signature =
            device.create_root_signature(point_root_signature_desc,
                                         t12::ERootSignatureVersion::V1)?;

        let mut point_input_layout_desc = t12::SInputLayoutDesc::create(&[
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

        let point_vertblob = t12::read_file_to_blob("shaders_built/point_vertex.cso")?;
        let point_pixelblob = t12::read_file_to_blob("shaders_built/point_pixel.cso")?;

        let point_vert_byte_code = t12::SShaderBytecode::create(point_vertblob);
        let point_pixel_byte_code = t12::SShaderBytecode::create(point_pixelblob);

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        let point_pipeline_state_stream = SPointPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&point_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut point_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Point,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&point_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&point_pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let point_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&point_pipeline_state_stream);
        let point_pipeline_state = device
            .raw()
            .create_pipeline_state(&point_pipeline_state_stream_desc)?;

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
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                }),
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
        // INSTANCE MESH pipeline state
        // =========================================================================================

        let instance_mesh_root_signature_flags = {
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
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let mut instance_mesh_root_signature_desc = t12::SRootSignatureDesc::new(instance_mesh_root_signature_flags);
        instance_mesh_root_signature_desc.parameters.push(vp_root_parameter);
        let instance_mesh_vp_root_param_idx = instance_mesh_root_signature_desc.parameters.len() - 1;

        let instance_mesh_root_signature =
            device.create_root_signature(instance_mesh_root_signature_desc,
                                         t12::ERootSignatureVersion::V1)?;

        let local_vert_slot = 0;
        let local_normal_slot = 1;
        let uvs_slot = 2;
        let instance_input_slot = 3;

        let mesh_input_elements = [
            shaderbindings::types::def_local_verts_input_element(local_vert_slot),
            shaderbindings::types::def_local_normals_input_element(local_normal_slot),
            shaderbindings::types::def_uvs_input_element(uvs_slot),
        ];

        let mut instance_mesh_input_layout_desc = t12::SInputLayoutDesc::create(&[
            mesh_input_elements[0],
            mesh_input_elements[1],
            mesh_input_elements[2],
            t12::SInputElementDesc::create(
                "INSTANCESCALE",
                0,
                t12::EDXGIFormat::R32Float,
                instance_input_slot,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerInstanceData,
                1,
            ),
            t12::SInputElementDesc::create(
                "INSTANCEPOSITION",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                instance_input_slot,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerInstanceData,
                1,
            ),
            t12::SInputElementDesc::create(
                "COLOR",
                0,
                t12::EDXGIFormat::R32G32B32A32Float,
                instance_input_slot,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerInstanceData,
                1,
            ),
        ]);

        let instance_mesh_vertblob = t12::read_file_to_blob("shaders_built/instance_mesh_vertex.cso")?;
        let instance_mesh_pixelblob = t12::read_file_to_blob("shaders_built/instance_mesh_pixel.cso")?;

        let instance_mesh_vert_byte_code = t12::SShaderBytecode::create(instance_mesh_vertblob);
        let instance_mesh_pixel_byte_code = t12::SShaderBytecode::create(instance_mesh_pixelblob);

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        let instance_mesh_pipeline_state_stream = SSpherePipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&instance_mesh_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut instance_mesh_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&instance_mesh_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&instance_mesh_pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let instance_mesh_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&instance_mesh_pipeline_state_stream);
        let instance_mesh_pipeline_state = device
            .raw()
            .create_pipeline_state(&instance_mesh_pipeline_state_stream_desc)?;

        // -- sphere mesh
        let sphere_mesh = SModel::new_from_obj("assets/debug_unit_sphere.obj", mesh_loader, texture_loader, 1.0, false)?.mesh;

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
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() / size_of::<f32>()) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Vertex,
        };
        let color_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 1,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Vec4>() / size_of::<f32>()) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        let mut mesh_root_signature_desc = t12::SRootSignatureDesc::new(mesh_root_signature_flags);
        mesh_root_signature_desc.parameters.push(mvp_root_parameter);
        let mesh_mvp_root_param_idx = mesh_root_signature_desc.parameters.len() - 1;
        mesh_root_signature_desc.parameters.push(color_root_parameter);
        let mesh_color_root_param_idx = mesh_root_signature_desc.parameters.len() - 1;
        let mesh_root_signature = device.create_root_signature(
            mesh_root_signature_desc, t12::ERootSignatureVersion::V1)?;

        // -- $$$FRK(TODO): wrong shader here!
        let mut mesh_input_layout_desc = shaderbindings::SVertexHLSL::input_layout_desc();

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

        let allocator = SYSTEM_ALLOCATOR();

        Ok(Self{
            point_pipeline_state,
            point_root_signature,
            point_vp_root_param_idx,
            _point_vert_byte_code: point_vert_byte_code,
            _point_pixel_byte_code: point_pixel_byte_code,
            points: SVec::new(&allocator, 1024, 0)?,
            point_vertex_buffer_intermediate_resource: [None, None],
            point_vertex_buffer_resource: [None, None],
            point_vertex_buffer_view: [None, None],

            line_pipeline_state,
            line_root_signature,
            line_vp_root_param_idx,
            _line_vert_byte_code: line_vert_byte_code,
            _line_pixel_byte_code: line_pixel_byte_code,
            lines: SVec::new(&allocator, 1024, 0)?,
            line_vertex_buffer_intermediate_resource: [None, None],
            line_vertex_buffer_resource: [None, None],
            line_vertex_buffer_view: [None, None],

            instance_mesh_pipeline_state,
            instance_mesh_root_signature,
            instance_mesh_vp_root_param_idx,
            _instance_mesh_vert_byte_code: instance_mesh_vert_byte_code,
            _instance_mesh_pixel_byte_code: instance_mesh_pixel_byte_code,

            spheres: SVec::new(&allocator, 1024, 0)?,
            sphere_mesh,
            sphere_instance_buffer_intermediate_resource: [None, None],
            sphere_instance_buffer_resource: [None, None],
            sphere_instance_buffer_view: [None, None],

            mesh_pipeline_state,
            mesh_root_signature,
            mesh_mvp_root_param_idx,
            mesh_color_root_param_idx,
            _mesh_vert_byte_code: mesh_vert_byte_code,
            _mesh_pixel_byte_code: mesh_pixel_byte_code,

            models: SVec::new(&allocator, 1024, 0)?,

            next_token: 1,
        })
    }

    pub fn get_token(&mut self) -> SToken {
        let result = SToken::new(self.next_token);
        self.next_token += 1;
        result
    }

    pub fn clear_token(&mut self, token: SToken) {
        macro_rules! clear_table {
            ($table:ident) => {
                let mut i = 0;
                while i < self.$table.len() {
                    if self.$table[i].token == token {
                        self.$table.swap_remove(i);
                    }
                    else {
                        i += 1;
                    }
                }
            }
        }

        clear_table!(points);
        clear_table!(lines);
        clear_table!(spheres);
        clear_table!(models);
    }

    pub fn draw_model(&mut self, model: &SModel, location: &STransform, over_world: bool) {
        assert!(model.diffuse_texture.is_none());

        self.models.push(STempModel{
            model: model.clone(),
            location: location.clone(),
            over_world,
            token: SToken::default(),
        });
    }

    #[allow(dead_code)]
    pub fn draw_point(&mut self, p: &Vec3, color: &Vec3, over_world: bool) {
        self.points.push(SPoint {
            p: p.clone(),
            colour: color.clone(),
            over_world,
            token: SToken::default(),
        });
    }

    pub fn draw_line(&mut self, start: &Vec3, end: &Vec3, color: &Vec4, over_world: bool, token: Option<SToken>) {
        self.lines.push(SLine {
            start: start.clone(),
            end: end.clone(),
            colour: color.clone(),
            over_world,
            token: token.unwrap_or(SToken::default()),
        });
    }

    pub fn draw_sphere(&mut self, pos: &Vec3, scale: f32, color: &Vec4, over_world: bool, token: Option<SToken>) {
        self.spheres.push(SSphere {
            scale,
            pos: pos.clone(),
            colour: color.clone(),
            over_world,
            token: token.unwrap_or(SToken::default()),
        });
    }

    pub fn draw_aabb(&mut self, aabb: &SAABB, color: &Vec4, over_world: bool) {
        let verts = [
            Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
        ];

        self.draw_line(&verts[0], &verts[1], color, over_world, None);
        self.draw_line(&verts[1], &verts[3], color, over_world, None);
        self.draw_line(&verts[3], &verts[2], color, over_world, None);
        self.draw_line(&verts[2], &verts[0], color, over_world, None);
        self.draw_line(&verts[0+4], &verts[1+4], color, over_world, None);
        self.draw_line(&verts[1+4], &verts[3+4], color, over_world, None);
        self.draw_line(&verts[3+4], &verts[2+4], color, over_world, None);
        self.draw_line(&verts[2+4], &verts[0+4], color, over_world, None);
        self.draw_line(&verts[0], &verts[0+4], color, over_world, None);
        self.draw_line(&verts[1], &verts[1+4], color, over_world, None);
        self.draw_line(&verts[3], &verts[3+4], color, over_world, None);
        self.draw_line(&verts[2], &verts[2+4], color, over_world, None);
    }

    pub fn clear_tables_without_tokens(&mut self) {
        let def_token = SToken::default();

        macro_rules! clear_table {
            ($table:ident) => {
                let mut i = 0;
                while i < self.$table.len() {
                    if self.$table[i].token == def_token {
                        self.$table.swap_remove(i);
                    }
                    else {
                        i += 1;
                    }
                }
            }
        }

        clear_table!(points);
        clear_table!(lines);
        clear_table!(spheres);
        clear_table!(models);
    }
}

impl super::SRender {
    pub fn render_temp_in_world(&mut self, context: &SRenderContext) -> Result<(), &'static str> {
        self.render_temp_points(context, true)?;
        self.render_temp_lines(context, true)?;
        self.render_temp_spheres(context, true)?;
        self.render_temp_models(context, true)?;

        Ok(())
    }

    pub fn render_temp_over_world(&mut self, context: &SRenderContext) -> Result<(), &'static str> {
        self.render_temp_points(context, false)?;
        self.render_temp_lines(context, false)?;
        self.render_temp_spheres(context, false)?;
        self.render_temp_models(context, false)?;

        Ok(())
    }

    pub fn render_temp_points(&mut self, context: &SRenderContext, in_world: bool) -> Result<(), &'static str> {
        // A very basic test
        /*
        self.temp().draw_point(
            &Vec3::new(0.0, 4.0, 0.0),
            &Vec3::new(1.0, 0.0, 0.0),
            true,
        );
        */

        if self.render_temp.points.len() == 0 {
            return Ok(());
        }

        // -- create/upload vertex buffer
        // -- must match SDebugLineShaderVert in debug_line_vertex.hlsl
        #[repr(C)]
        struct SDebugPointShaderVert {
            pos: [f32; 3],
            colour: [f32; 3], // no alpha, otherwise points would vanish
        }
        impl SDebugPointShaderVert {
            fn new(pos: &Vec3, colour: &Vec3) -> Self {
                Self {
                    pos: [pos.x, pos.y, pos.z],
                    colour: [colour.x, colour.y, colour.z],
                }
            }
        }

        // -- generate data and copy to GPU
        // -- $$$FRK(TODO): move this step to earlier in render?
        let points_to_draw = STACK_ALLOCATOR.with(|sa| -> Result<bool, &'static str> {
            let tr = &mut self.render_temp;

            let mut vertex_buffer_data = SVec::new(
                &sa.as_ref(),
                tr.points.len(),
                0,
            )?;

            let over_world = !in_world;
            for point in tr.points.as_slice() {
                if point.over_world != over_world { continue; }

                vertex_buffer_data.push(SDebugPointShaderVert::new(&point.p, &point.colour));
            }

            if vertex_buffer_data.len() == 0 {
                return Ok(false);
            }

            let mut handle = self.copy_command_pool.alloc_list()?;
            let mut copy_command_list = self.copy_command_pool.get_list(&handle)?;

            let vert_buffer_resource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copy_command_list.update_buffer_resource(
                    self.device.deref(),
                    vertex_buffer_data.as_slice(),
                    vertbufferflags
                )?
            };
            let vertex_buffer_view = vert_buffer_resource
                .destinationresource.raw
                .create_vertex_buffer_view()?;

            drop(copy_command_list);
            let fence_val = self.copy_command_pool.execute_and_free_list(&mut handle)?;
            drop(handle);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_pool.gpu_wait(
                self.copy_command_pool.get_internal_fence(),
                fence_val,
            )?;

            tr.point_vertex_buffer_intermediate_resource[context.current_back_buffer_index] =
                Some(vert_buffer_resource.intermediateresource.raw);
            tr.point_vertex_buffer_resource[context.current_back_buffer_index] =
                Some(vert_buffer_resource.destinationresource.raw);
            tr.point_vertex_buffer_view[context.current_back_buffer_index] = Some(vertex_buffer_view);

            Ok(true)
        })?;

        if !points_to_draw {
            return Ok(());
        }

        // -- set up pipeline and render points
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        list.set_pipeline_state(&self.render_temp.point_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.point_root_signature.raw());

        // -- setup rasterizer state
        list.rs_set_viewports(&[&context.viewport]);

        // -- setup the output merger
        let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&context.render_target_view], false, &depth_texture_view);

        list.set_graphics_root_32_bit_constants(self.render_temp.point_vp_root_param_idx,
                                                &context.view_projection_matrix, 0);

        // -- set up input assembler
        list.ia_set_primitive_topology(t12::EPrimitiveTopology::PointList);
        let vert_buffer_view = self.render_temp.point_vertex_buffer_view[context.current_back_buffer_index].
            as_ref().expect("should have generated resource earlier in this function");
        list.ia_set_vertex_buffers(0, &[vert_buffer_view]);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        for i in 0..self.render_temp.points.len() {
            list.draw_instanced(1, 1, i as u32, 0);
        }

        // -- execute on the queue
        drop(list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }


    pub fn render_temp_lines(&mut self, context: &SRenderContext, in_world: bool) -> Result<(), &'static str> {
        // A very basic test
        /*
        self.render_temp.lines.push(SLine{
            start: Vec3::new(-5.0, 2.0, 0.0),
            end: Vec3::new(5.0, 2.0, 0.0),
            colour: Vec4::new(1.0, 0.0, 0.0, 1.0),
            over_world: true,
            token: SToken::default(),
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
            colour: [f32; 4],
        }
        impl SDebugLineShaderVert {
            fn new(pos: &Vec3, colour: &Vec4) -> Self {
                Self {
                    pos: [pos.x, pos.y, pos.z],
                    colour: [colour.x, colour.y, colour.z, colour.w],
                }
            }
        }

        // -- generate data and copy to GPU
        // -- $$$FRK(TODO): move this step to earlier in render?
        let lines_to_draw = STACK_ALLOCATOR.with(|sa| -> Result<bool, &'static str> {
            let tr = &mut self.render_temp;

            let mut vertex_buffer_data = SVec::new(
                &sa.as_ref(),
                tr.lines.len() * 2,
                0,
            )?;

            let over_world = !in_world;

            for line in tr.lines.as_slice() {
                if line.over_world != over_world { continue; }

                vertex_buffer_data.push(SDebugLineShaderVert::new(&line.start, &line.colour));
                vertex_buffer_data.push(SDebugLineShaderVert::new(&line.end, &line.colour));
            }

            if vertex_buffer_data.len() == 0 {
                return Ok(false);
            }

            let mut handle = self.copy_command_pool.alloc_list()?;
            let mut copy_command_list = self.copy_command_pool.get_list(&handle)?;

            let vert_buffer_resource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copy_command_list.update_buffer_resource(
                    self.device.deref(),
                    vertex_buffer_data.as_slice(),
                    vertbufferflags
                )?
            };
            let vertex_buffer_view = vert_buffer_resource
                .destinationresource.raw
                .create_vertex_buffer_view()?;

            drop(copy_command_list);
            let fence_val = self.copy_command_pool.execute_and_free_list(&mut handle)?;
            drop(handle);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_pool.gpu_wait(
                self.copy_command_pool.get_internal_fence(),
                fence_val,
            )?;

            tr.line_vertex_buffer_intermediate_resource[context.current_back_buffer_index] =
                Some(vert_buffer_resource.intermediateresource.raw);
            tr.line_vertex_buffer_resource[context.current_back_buffer_index] =
                Some(vert_buffer_resource.destinationresource.raw);
            tr.line_vertex_buffer_view[context.current_back_buffer_index] = Some(vertex_buffer_view);

            Ok(true)
        })?;

        if !lines_to_draw {
            return Ok(());
        }

        // -- set up pipeline and render lines
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        list.set_pipeline_state(&self.render_temp.line_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.line_root_signature.raw());

        // -- setup rasterizer state
        list.rs_set_viewports(&[&context.viewport]);

        // -- setup the output merger
        let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&context.render_target_view], false, &depth_texture_view);

        list.set_graphics_root_32_bit_constants(self.render_temp.line_vp_root_param_idx,
                                                &context.view_projection_matrix, 0);

        // -- set up input assembler
        list.ia_set_primitive_topology(t12::EPrimitiveTopology::LineList);
        let vert_buffer_view = self.render_temp.line_vertex_buffer_view[context.current_back_buffer_index].
            as_ref().expect("should have generated resource earlier in this function");
        list.ia_set_vertex_buffers(0, &[vert_buffer_view]);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        // -- $$$FRK(TODO): this should be doable in one draw call
        for i in 0..self.render_temp.lines.len() {
            list.draw_instanced(2, 1, (i * 2) as u32, 0);
        }

        // -- execute on the queue
        drop(list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }

    pub fn render_temp_spheres(&mut self, context: &SRenderContext, in_world: bool) -> Result<(), &'static str> {
        // A very basic test
        /*
        self.temp().draw_sphere(&Vec3::new(-1.0, 4.0, 0.0), 0.2, &Vec4::new(1.0, 0.0, 0.0, 0.5), false, None);
        self.temp().draw_sphere(&Vec3::new(0.0, 4.0, 0.0), 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.5), false, None);
        self.temp().draw_sphere(&Vec3::new(1.0, 4.0, 0.0), 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.5), false, None);
        self.temp().draw_sphere(&Vec3::new(2.0, 4.0, 0.0), 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.5), false, None);
        */

        if self.render_temp.spheres.len() == 0 {
            return Ok(());
        }

        // -- create/upload instance buffer
        #[repr(C)]
        struct SDebugSphereShaderInstance {
            scale: f32,
            pos: [f32; 3],
            colour: [f32; 4],
        }
        impl SDebugSphereShaderInstance {
            fn new(scale: f32, pos: &Vec3, colour: &Vec4) -> Self {
                Self {
                    scale,
                    pos: [pos.x, pos.y, pos.z],
                    colour: [colour.x, colour.y, colour.z, colour.w],
                }
            }
        }

        // -- generate data and copy to GPU
        // -- $$$FRK(TODO): move this step to earlier in render?
        let sphere_count = STACK_ALLOCATOR.with(|sa| -> Result<usize, &'static str> {
            let tr = &mut self.render_temp;

            let mut instance_buffer_data = SVec::new(
                &sa.as_ref(),
                tr.spheres.len(),
                0,
            )?;

            let over_world = !in_world;
            for sphere in tr.spheres.as_slice() {
                if sphere.over_world != over_world { continue; }
                instance_buffer_data.push(SDebugSphereShaderInstance::new(sphere.scale, &sphere.pos, &sphere.colour));
            }

            if instance_buffer_data.len() == 0 {
                return Ok(0);
            }

            let mut handle = self.copy_command_pool.alloc_list()?;
            let mut copy_command_list = self.copy_command_pool.get_list(&handle)?;

            let instance_buffer_resource = {
                let instance_buffer_flags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copy_command_list.update_buffer_resource(
                    self.device.deref(),
                    instance_buffer_data.as_slice(),
                    instance_buffer_flags
                )?
            };
            let instance_buffer_view = instance_buffer_resource
                .destinationresource.raw
                .create_vertex_buffer_view()?;

            drop(copy_command_list);
            let fence_val = self.copy_command_pool.execute_and_free_list(&mut handle)?;
            drop(handle);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_pool.gpu_wait(
                self.copy_command_pool.get_internal_fence(),
                fence_val,
            )?;

            tr.sphere_instance_buffer_intermediate_resource[context.current_back_buffer_index] =
                Some(instance_buffer_resource.intermediateresource.raw);
            tr.sphere_instance_buffer_resource[context.current_back_buffer_index] =
                Some(instance_buffer_resource.destinationresource.raw);
            tr.sphere_instance_buffer_view[context.current_back_buffer_index] = Some(instance_buffer_view);

            Ok(instance_buffer_data.len())
        })?;

        if sphere_count == 0 {
            return Ok(());
        }

        // -- set up pipeline and render lines
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        list.set_pipeline_state(&self.render_temp.instance_mesh_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.instance_mesh_root_signature.raw());

        // -- setup rasterizer state
        list.rs_set_viewports(&[&context.viewport]);

        // -- setup the output merger
        let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&context.render_target_view], false, &depth_texture_view);

        list.set_graphics_root_32_bit_constants(self.render_temp.instance_mesh_vp_root_param_idx,
                                                &context.view_projection_matrix, 0);

        // -- set up input assembler
        list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
        let local_verts_vbv = self.mesh_loader.local_verts_vbv(self.render_temp.sphere_mesh);
        let local_normals_vbv = self.mesh_loader.local_normals_vbv(self.render_temp.sphere_mesh);
        let uvs_vbv = self.mesh_loader.uvs_vbv(self.render_temp.sphere_mesh);
        let indices_ibv = self.mesh_loader.indices_ibv(self.render_temp.sphere_mesh);
        let instance_buffer_view = self.render_temp.sphere_instance_buffer_view[context.current_back_buffer_index].
            as_ref().expect("should have generated resource earlier in this function");

        list.ia_set_vertex_buffers(0, &[local_verts_vbv, local_normals_vbv, uvs_vbv, &instance_buffer_view]);
        list.ia_set_index_buffer(&indices_ibv);

        // -- set up rasterizer
        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        // -- draw call
        let index_count = self.mesh_loader.index_count(self.render_temp.sphere_mesh);
        list.draw_indexed_instanced(index_count as u32, sphere_count as u32, 0, 0, 0);

        // -- execute on the queue
        drop(list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }

    pub fn render_temp_models(&mut self, context: &SRenderContext, in_world: bool) -> Result<(), &'static str> {
        if self.render_temp.models.len() == 0 {
            return Ok(());
        }

        // -- set up pipeline and render models
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        list.set_pipeline_state(&self.render_temp.mesh_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.render_temp.mesh_root_signature.raw());

        // -- setup rasterizer state
        list.rs_set_viewports(&[&context.viewport]);

        // -- setup the output merger
        let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&context.render_target_view], false, &depth_texture_view);

        list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        let over_world = !in_world;

        for model in self.render_temp.models.as_slice() {
            if model.over_world != over_world { continue; }

            let model_matrix = model.location.as_mat4();
            let mvp = context.view_projection_matrix * model_matrix;

            list.set_graphics_root_32_bit_constants(self.render_temp.mesh_mvp_root_param_idx,
                                                    &mvp, 0);
            list.set_graphics_root_32_bit_constants(self.render_temp.mesh_color_root_param_idx,
                                                    &model.model.diffuse_colour, 0);

            let local_verts_vbv = self.mesh_loader.local_verts_vbv(model.model.mesh);
            let local_normals_vbv = self.mesh_loader.local_normals_vbv(model.model.mesh);
            let uvs_vbv = self.mesh_loader.uvs_vbv(model.model.mesh);

            list.ia_set_vertex_buffers(0, &[local_verts_vbv, local_normals_vbv, uvs_vbv]);

            self.mesh_loader.set_index_buffer_and_draw(model.model.mesh, &mut list)?;
        }

        // -- execute on the queue
        drop(list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }
}