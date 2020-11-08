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
use model::{SModel, SMeshLoader, STextureLoader, SMeshHandle};
use utils::{STransform, SAABB};

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SSkinnedPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

pub struct SRenderSkinned<'a> {
    pipeline_state: t12::SPipelineState,
    root_signature: n12::SRootSignature,
    _vert_byte_code: t12::SShaderBytecode,
    _pixel_byte_code: t12::SShaderBytecode,
}

impl<'a> SRenderSkinned<'a> {

    pub fn new(device: &n12::SDevice, mesh_loader: &mut SMeshLoader, texture_loader: &mut STextureLoader) -> Result<Self, &'static str> {
        let root_signature_flags = {
            use t12::ERootSignatureFlags::*;

            t12::SRootSignatureFlags::create(&[
                AllowInputAssemblerInputLayout,
                DenyHullShaderRootAccess,
                DenyDomainShaderRootAccess,
                DenyGeometryShaderRootAccess,
                DenyPixelShaderRootAccess,
            ])
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

    }

impl<'a> super::SRender<'a> {
}