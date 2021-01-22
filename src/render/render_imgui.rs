// -- std includes
use std::mem::{size_of};
use std::ops::{Deref};

// -- crate includes
use arrayvec::{ArrayVec};
use imgui;
use crate::math::{Mat4};

use crate::allocate::{SYSTEM_ALLOCATOR};
use crate::collections::{SVec};
use crate::model::{STextureLoader, STextureHandle};
use crate::niced3d12 as n12;
use crate::typeyd3d12 as t12;

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SImguiPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    blend_state: n12::SPipelineStateStreamBlendDesc,
    depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

pub(super) struct SRenderImgui {
    font_texture: STextureHandle,
    font_texture_id: imgui::TextureId,
    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,
    orthomat_root_param_idx: usize,
    texture_descriptor_table_param_idx: usize,
    _vert_byte_code: t12::SShaderBytecode,
    _pixel_byte_code: t12::SShaderBytecode,
    vert_buffer_resources: [SVec::<n12::SResource>; 2],
    int_vert_buffer_resources: [SVec::<n12::SResource>; 2],
    vert_buffer_views: [SVec::<t12::SVertexBufferView>; 2],
    index_buffer_resources: [SVec::<n12::SResource>; 2],
    int_index_buffer_resources: [SVec::<n12::SResource>; 2],
    index_buffer_views: [SVec::<t12::SIndexBufferView>; 2],
}

impl SRenderImgui {
    pub fn new(imgui_ctxt: &mut imgui::Context, texture_loader: &mut STextureLoader, device: &n12::SDevice) -> Result<Self, &'static str> {
        // -- set up font
        let font_size = 13.0 as f32;
        imgui_ctxt.fonts().add_font(&[
            imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..imgui::FontConfig::default()
                }),
            },
        ]);

        let mut fonts = imgui_ctxt.fonts();
        let font_atlas_texture = fonts.build_rgba32_texture();
        let font_texture = texture_loader.create_texture_rgba32_from_bytes(
            font_atlas_texture.width,
            font_atlas_texture.height,
            font_atlas_texture.data,
        )?;
        drop(fonts);

        let orthomat_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants(
                t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() * 3 / 4) as u32,
                }),
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let texture_root_parameter = {
            let descriptor_range = t12::SDescriptorRange {
                range_type: t12::EDescriptorRangeType::SRV,
                num_descriptors: 1,
                base_shader_register: 0,
                register_space: 0,
                offset_in_descriptors_from_table_start: t12::EDescriptorRangeOffset::EAppend,
            };

            let mut root_descriptor_table = t12::SRootDescriptorTable::new();
            root_descriptor_table
                .descriptor_ranges
                .push(descriptor_range);

            t12::SRootParameter {
                type_: t12::ERootParameterType::DescriptorTable(root_descriptor_table),
                shader_visibility: t12::EShaderVisibility::Pixel,
            }
        };

        let sampler = t12::SStaticSamplerDesc {
            filter: t12::EFilter::MinMagMipPoint,
            address_u: t12::ETextureAddressMode::Border,
            address_v: t12::ETextureAddressMode::Border,
            address_w: t12::ETextureAddressMode::Border,
            mip_lod_bias: 0.0,
            max_anisotropy: 0,
            comparison_func: t12::EComparisonFunc::Never,
            border_color: t12::EStaticBorderColor::OpaqueWhite,
            min_lod: 0.0,
            max_lod: std::f32::MAX,
            shader_register: 0,
            register_space: 0,
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        let root_signature_flags = t12::SRootSignatureFlags::create(&[
            t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
            t12::ERootSignatureFlags::DenyHullShaderRootAccess,
            t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
            t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        ]);

        let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
        root_signature_desc.parameters.push(orthomat_root_parameter);
        let orthomat_root_param_idx = root_signature_desc.parameters.len() - 1;
        root_signature_desc.parameters.push(texture_root_parameter);
        let texture_descriptor_table_param_idx = root_signature_desc.parameters.len() - 1;
        root_signature_desc.static_samplers.push(sampler);

        let root_signature =
            device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

        // -- load shaders
        let vertblob = t12::read_file_to_blob("shaders_built/imgui_vertex.cso")?;
        let pixelblob = t12::read_file_to_blob("shaders_built/imgui_pixel.cso")?;

        let vert_byte_code = t12::SShaderBytecode::create(vertblob);
        let pixel_byte_code = t12::SShaderBytecode::create(pixelblob);

        let depth_stencil_desc = t12::SDepthStencilDesc {
            depth_enable: false,
            ..Default::default()
        };

        let mut input_layout_desc = t12::SInputLayoutDesc::create(&[
            t12::SInputElementDesc::create(
                "POSITION",
                0,
                t12::EDXGIFormat::R32G32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "TEXCOORD",
                0,
                t12::EDXGIFormat::R32G32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "COLOR",
                0,
                t12::EDXGIFormat::R32UINT,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ]);

        let mut blend_desc = t12::SBlendDesc::default();
        blend_desc.render_target_blend_desc[0].blend_enable = true;
        blend_desc.render_target_blend_desc[0].src_blend = t12::EBlend::SrcAlpha;
        blend_desc.render_target_blend_desc[0].dest_blend = t12::EBlend::InvSrcAlpha;
        blend_desc.render_target_blend_desc[0].src_blend_alpha = t12::EBlend::One;
        blend_desc.render_target_blend_desc[0].dest_blend_alpha = t12::EBlend::InvSrcAlpha;

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        let pipeline_state_stream = SImguiPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
            blend_state: n12::SPipelineStateStreamBlendDesc::create(blend_desc),
            depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc::create(depth_stencil_desc),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
        let pipeline_state = device
            .raw()
            .create_pipeline_state(&pipeline_state_stream_desc)?;

        let allocator = SYSTEM_ALLOCATOR();

        let vert_buffer_resources = [
            SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];
        let int_vert_buffer_resources = [
            SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];
        let vert_buffer_views = [SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];
        let index_buffer_resources = [SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];
        let int_index_buffer_resources = [SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];
        let index_buffer_views = [SVec::new(&allocator, 128, 0)?,
            SVec::new(&allocator, 128, 0)?,
        ];

        Ok(Self {
            font_texture,
            font_texture_id: imgui_ctxt.fonts().tex_id,
            root_signature,
            pipeline_state,
            orthomat_root_param_idx,
            texture_descriptor_table_param_idx,
            _vert_byte_code: vert_byte_code,
            _pixel_byte_code: pixel_byte_code,
            vert_buffer_resources,
            int_vert_buffer_resources,
            vert_buffer_views,
            index_buffer_resources,
            int_index_buffer_resources,
            index_buffer_views,
        })
    }

    fn get_imgui_texture(&self, texture_id: imgui::TextureId) -> STextureHandle {
        if texture_id == self.font_texture_id {
            return self.font_texture;
        }

        panic!("We don't have any other textures!!!!");
    }
}

impl super::SRender {
    pub fn setup_imgui_draw_data_resources(&mut self, window: &n12::SD3D12Window, draw_data: &imgui::DrawData) -> Result<(), &'static str> {
        let ri = &mut self.render_imgui;

        let backbufferidx = window.currentbackbufferindex();
        ri.vert_buffer_resources[backbufferidx].clear();
        ri.int_vert_buffer_resources[backbufferidx].clear();
        ri.vert_buffer_views[backbufferidx].clear();
        ri.index_buffer_resources[backbufferidx].clear();
        ri.int_index_buffer_resources[backbufferidx].clear();
        ri.index_buffer_views[backbufferidx].clear();

        let mut handle = self.copy_command_pool.alloc_list()?;
        let mut copy_command_list = self.copy_command_pool.get_list(&handle)?;

        for draw_list in draw_data.draw_lists() {
            let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
                let mut vertbufferresource = {
                    let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                    copy_command_list.update_buffer_resource(
                        self.device.deref(),
                        draw_list.vtx_buffer(),
                        vertbufferflags
                    )?
                };
                let vertexbufferview = vertbufferresource
                    .destinationresource.raw
                    .create_vertex_buffer_view()?;

                let mut indexbufferresource = {
                    let indexbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                    copy_command_list.update_buffer_resource(
                        self.device.deref(),
                        draw_list.idx_buffer(),
                        indexbufferflags
                    )?
                };
                let indexbufferview = indexbufferresource
                    .destinationresource.raw
                    .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

                unsafe {
                    vertbufferresource.destinationresource.raw.set_debug_name("imgui vert dest");
                    vertbufferresource.intermediateresource.raw.set_debug_name("imgui vert inter");
                    indexbufferresource.destinationresource.raw.set_debug_name("imgui index dest");
                    indexbufferresource.intermediateresource.raw.set_debug_name("imgui index inter");
                }

                (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
            };

            // -- save the data until the next frame
            ri.vert_buffer_resources[backbufferidx].push(vertbufferresource.destinationresource.raw);
            ri.int_vert_buffer_resources[backbufferidx].push(vertbufferresource.intermediateresource.raw);
            ri.vert_buffer_views[backbufferidx].push(vertexbufferview);
            ri.index_buffer_resources[backbufferidx].push(indexbufferresource.destinationresource.raw);
            ri.int_index_buffer_resources[backbufferidx].push(indexbufferresource.intermediateresource.raw);
            ri.index_buffer_views[backbufferidx].push(indexbufferview);
        }
        drop(copy_command_list);
        self.copy_command_pool.execute_and_free_list(&mut handle)?;
        drop(handle);


        // -- wait on copies on direct queue, then transition resources
        let mut handle  = self.direct_command_pool.alloc_list()?;
        let mut direct_command_list = self.direct_command_pool.get_list(&handle)?;

        // -- have the direct queue wait on the copy upload to complete
        self.direct_command_pool.gpu_wait(
            self.copy_command_pool.get_internal_fence(),
            self.copy_command_pool.get_internal_fence().last_signalled_value(),
        )?;

        for dest_resource in ri.vert_buffer_resources[backbufferidx].as_slice() {
            direct_command_list.transition_resource(
                &dest_resource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::VertexAndConstantBuffer,
            )?;
        }
        for dest_resource in ri.index_buffer_resources[backbufferidx].as_slice() {
            direct_command_list.transition_resource(
                &dest_resource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::IndexBuffer,
            )?;
        }

        drop(direct_command_list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }

    pub fn render_imgui(&mut self, window: &n12::SD3D12Window, draw_data: &imgui::DrawData) -> Result<(), &'static str> {
        let ri = &mut self.render_imgui;

        let backbufferidx = window.currentbackbufferindex();

        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        // -- set up pipeline
        list.set_pipeline_state(&ri.pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&ri.root_signature.raw());

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
        let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);
        list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

        self.cbv_srv_uav_heap.with_raw_heap(|rh| {
            list.set_descriptor_heaps(&[rh]);
        });

        let ortho_matrix: Mat4 = {
            let znear = 0.0;
            let zfar = 1.0;

            let left = draw_data.display_pos[0];
            let right = draw_data.display_pos[0] + draw_data.display_size[0];
            let bottom = draw_data.display_pos[1] + draw_data.display_size[1];
            let top = draw_data.display_pos[1];

            Mat4::new_orthographic(left, right, bottom, top, znear, zfar)
        };

        list.set_graphics_root_32_bit_constants(ri.orthomat_root_param_idx, &ortho_matrix, 0);

        for (i, draw_list) in draw_data.draw_lists().enumerate() {

            // -- set up input assembler
            list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
            list.ia_set_vertex_buffers(0, &[&ri.vert_buffer_views[backbufferidx][i]]);
            list.ia_set_index_buffer(&ri.index_buffer_views[backbufferidx][i]);

            for cmd in draw_list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params:
                            imgui::DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                            },
                    } => {
                        if clip_rect[0] > (window.width() as f32) || clip_rect[1] > (window.height() as f32) ||
                           clip_rect[2] < 0.0 || clip_rect[3] < 0.0 {
                            continue;
                        }

                        let scissorrect = t12::SRect {
                            left: f32::max(0.0, clip_rect[0]).floor() as i32,
                            right: f32::min(clip_rect[2], window.width() as f32).floor() as i32,
                            top: f32::max(0.0, clip_rect[1]).floor() as i32,
                            bottom: f32::min(clip_rect[3], window.height() as f32).floor() as i32,
                        };

                        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

                        let texture = ri.get_imgui_texture(texture_id);
                        list.set_graphics_root_descriptor_table(
                            ri.texture_descriptor_table_param_idx,
                            &self.texture_loader.texture_gpu_descriptor(texture).unwrap(),
                        );

                        list.draw_indexed_instanced(count as u32, 1, idx_offset as u32, vtx_offset as i32, 0);
                    },
                    imgui::DrawCmd::ResetRenderState => {},
                    imgui::DrawCmd::RawCallback{..} => {},
                }
            }
        }

        // -- execute on the queue
        drop(list);
        assert_eq!(window.currentbackbufferindex(), backbufferidx);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        Ok(())
    }
}