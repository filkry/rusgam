// -- std includes
use std::cell::RefCell;
use std::io::Write;
use std::mem::{size_of};
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

// -- crate includes
use arrayvec::{ArrayVec};
use serde::{Serialize, Deserialize};
use glm::{Vec3, Mat4};

use niced3d12 as n12;
use typeyd3d12 as t12;
use allocate::{SMemVec, STACK_ALLOCATOR, SYSTEM_ALLOCATOR};
use collections::{SPoolHandle};
use model;
use model::{SModel, SMeshLoader, STextureLoader};
use safewindows;
use shadowmapping;
use rustywindows;
use utils;
use utils::{STransform};

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SPipelineStateStream<'a> {
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
struct SImguiPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[derive(Serialize, Deserialize)]
struct SBuiltShaderMetadata {
    src_write_time: std::time::SystemTime,
}

pub struct SRender<'a> {
    factory: n12::SFactory,
    _adapter: n12::SAdapter, // -- maybe don't need to keep

    direct_command_pool: n12::SCommandListPool,
    copy_command_pool: n12::SCommandListPool,

    _depth_texture_resource: Option<n12::SResource>,
    _depth_texture_view: Option<n12::SDescriptorAllocatorAllocation>,

    // -- these depend on the heaps existing, so should be dropped first
    mesh_loader: SMeshLoader<'a>,
    texture_loader: STextureLoader,

    scissorrect: t12::SRect,
    fovy: f32,
    znear: f32,

    _vert_byte_code: t12::SShaderBytecode,
    _pixel_byte_code: t12::SShaderBytecode,

    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,

    shadow_mapping_pipeline: shadowmapping::SShadowMappingPipeline,

    // -- imgui stuff
    imgui_font_texture: SPoolHandle,
    imgui_font_texture_id: imgui::TextureId,
    imgui_root_signature: n12::SRootSignature,
    imgui_pipeline_state: t12::SPipelineState,
    imgui_orthomat_root_param_idx: usize,
    imgui_texture_descriptor_table_param_idx: usize,
    _imgui_vert_byte_code: t12::SShaderBytecode,
    _imgui_pixel_byte_code: t12::SShaderBytecode,
    imgui_vert_buffer_resources: SMemVec::<'a, n12::SResource>,
    imgui_vert_buffer_views: SMemVec::<'a, t12::SVertexBufferView>,
    imgui_index_buffer_resources: SMemVec::<'a, n12::SResource>,
    imgui_index_buffer_views: SMemVec::<'a, t12::SIndexBufferView>,

    frame_fence_values: [u64; 2],

    // -- these things need to drop last, due to Weak references to them
    dsv_heap: Rc<n12::SDescriptorAllocator>,
    srv_heap: Rc<n12::SDescriptorAllocator>,

    _copy_command_queue: Rc<RefCell<n12::SCommandQueue>>, // -- used in mesh/texture loader via Weak
    direct_command_queue: Rc<RefCell<n12::SCommandQueue>>,

    device: Rc<n12::SDevice>,
}

pub fn compile_shaders_if_changed() {
    let shaders = [
        ("vertex", "vs_6_0"),
        ("pixel", "ps_6_0"),
        ("shadow_vertex", "vs_6_0"),
        ("shadow_pixel", "ps_6_0"),
        ("imgui_vertex", "vs_6_0"),
        ("imgui_pixel", "ps_6_0"),
    ];

    for (shader_name, type_) in &shaders {
        let mut needs_build = false;

        STACK_ALLOCATOR.with(|sa| {
            let mut shader_src_path_string = SMemVec::<u8>::new(sa, 256, 0).unwrap();
            write!(&mut shader_src_path_string, "shaders/{}.hlsl", shader_name).unwrap();
            let shader_src_path = std::path::Path::new(shader_src_path_string.as_str());

            let mut built_shader_path_string = SMemVec::<u8>::new(sa, 256, 0).unwrap();
            write!(&mut built_shader_path_string, "shaders_built/{}.cso", shader_name).unwrap();
            let built_shader_path = std::path::Path::new(built_shader_path_string.as_str());

            let mut build_metadata_path_string = SMemVec::<u8>::new(sa, 256, 0).unwrap();
            write!(&mut build_metadata_path_string, "shaders_built/{}.shader_build_metadata", shader_name).unwrap();
            let build_metadata_path = std::path::Path::new(build_metadata_path_string.as_str());

            if !build_metadata_path.exists() {
                needs_build = true;
            }
            else {
                needs_build = true;

                /*
                let src_write_time = {
                    let src_file = std::fs::OpenOptions::new().read(true).open(shader_src_path).unwrap();
                    src_file.metadata().unwrap().modified().unwrap()
                };

                // -- $$$FRK(TODO): can I read into a custom buffer to not allocate from system heap?
                let build_shader_metadata_json_str = std::fs::read_to_string(build_metadata_path).unwrap();
                let build_shader_metadata : SBuiltShaderMetadata = serde_json::from_str(build_shader_metadata_json_str.as_str()).unwrap();

                if src_write_time != build_shader_metadata.src_write_time {
                    needs_build = true;
                }
                */
            }

            if needs_build {
                println!("Compiling shader {}...", shader_name);

                let mut command = std::process::Command::new("externals/dxc_2019-07-15/dxc.exe");

                command.arg("-E").arg("main")
                       .arg("-T").arg(type_)
                       .arg(shader_src_path)
                       .arg("-Fo").arg(built_shader_path);

                println!("   commmand: \"{:?}\"", command);

                command.status().expect("Failed to compile shader");

                let src_write_time = {
                    let src_file = std::fs::OpenOptions::new().read(true).open(shader_src_path).unwrap();
                    src_file.metadata().unwrap().modified().unwrap()
                };

                let built_shader_metadata = SBuiltShaderMetadata {
                    src_write_time,
                };

                let build_shader_metadata_json_str = serde_json::to_string(&built_shader_metadata).unwrap();

                let mut built_shader_file = std::fs::OpenOptions::new().create(true).write(true).open(build_metadata_path).unwrap();
                built_shader_file.write_all(build_shader_metadata_json_str.as_bytes()).unwrap();
            }
        });

    }
}

impl<'a> SRender<'a> {
    pub fn new(winapi: &rustywindows::SWinAPI, imgui_ctxt: &mut imgui::Context) -> Result<Self, &'static str> {
        // -- initialize debug
        let debuginterface = t12::SDebugInterface::new()?;
        debuginterface.enabledebuglayer();

        let mut factory = n12::SFactory::create()?;
        let mut adapter = factory.create_best_adapter()?;
        let device = Rc::new(adapter.create_device()?);

        // -- set up command queues
        let direct_command_queue = Rc::new(RefCell::new(
            device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Direct)?,
        ));
        unsafe { direct_command_queue.borrow_mut().set_debug_name("render direct queue"); }
        let mut direct_command_pool =
            n12::SCommandListPool::create(&device, Rc::downgrade(&direct_command_queue), &winapi.rawwinapi(), 1, 10)?;

        let dsv_heap = Rc::new(n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::DepthStencil,
            t12::SDescriptorHeapFlags::none(),
        )?);

        let srv_heap = Rc::new(n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess,
            t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::ShaderVisible),
        )?);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };

        let copy_command_queue = Rc::new(RefCell::new(
            device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Copy)?,
        ));
        unsafe { direct_command_queue.borrow_mut().set_debug_name("render copy queue"); }
        let copy_command_pool =
            n12::SCommandListPool::create(&device, Rc::downgrade(&copy_command_queue), &winapi.rawwinapi(), 1, 10)?;
        let mesh_loader = SMeshLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), 23948934, 1024)?;
        let mut texture_loader = STextureLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), Rc::downgrade(&srv_heap), 9323, 1024)?;

        // -- load shaders
        let vertblob = t12::read_file_to_blob("shaders_built/vertex.cso")?;
        let pixelblob = t12::read_file_to_blob("shaders_built/pixel.cso")?;

        let vert_byte_code = t12::SShaderBytecode::create(vertblob);
        let pixel_byte_code = t12::SShaderBytecode::create(pixelblob);

        // -- root signature stuff
        let mut input_layout_desc = model::model_per_vertex_input_layout_desc();

        let mvp_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() * 3 / 4) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let texture_metadata_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 1,
                    register_space: 0,
                    num_32_bit_values: (size_of::<model::STextureMetadata>() / 4) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Pixel,
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
                type_: t12::ERootParameterType::DescriptorTable,
                type_data: t12::ERootParameterTypeData::DescriptorTable {
                    table: root_descriptor_table,
                },
                shader_visibility: t12::EShaderVisibility::Pixel,
            }
        };

        let shadow_cube_root_parameter = {
            let descriptor_range = t12::SDescriptorRange {
                range_type: t12::EDescriptorRangeType::SRV,
                num_descriptors: 1,
                base_shader_register: 0,
                register_space: 1,
                offset_in_descriptors_from_table_start: t12::EDescriptorRangeOffset::EAppend,
            };

            let mut root_descriptor_table = t12::SRootDescriptorTable::new();
            root_descriptor_table
                .descriptor_ranges
                .push(descriptor_range);

            t12::SRootParameter {
                type_: t12::ERootParameterType::DescriptorTable,
                type_data: t12::ERootParameterTypeData::DescriptorTable {
                    table: root_descriptor_table,
                },
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

        let shadow_sampler = t12::SStaticSamplerDesc {
            filter: t12::EFilter::MinMagMipPoint,
            address_u: t12::ETextureAddressMode::Clamp,
            address_v: t12::ETextureAddressMode::Clamp,
            address_w: t12::ETextureAddressMode::Clamp,
            mip_lod_bias: 0.0,
            max_anisotropy: 0,
            comparison_func: t12::EComparisonFunc::Never,
            border_color: t12::EStaticBorderColor::OpaqueWhite,
            min_lod: 0.0,
            max_lod: 0.0,
            shader_register: 0,
            register_space: 1,
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        let root_signature_flags = t12::SRootSignatureFlags::create(&[
            t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
            t12::ERootSignatureFlags::DenyHullShaderRootAccess,
            t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
            t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        ]);

        let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
        root_signature_desc.parameters.push(mvp_root_parameter);
        root_signature_desc.parameters.push(texture_metadata_root_parameter);
        root_signature_desc.parameters.push(texture_root_parameter);
        root_signature_desc.parameters.push(shadow_cube_root_parameter);
        root_signature_desc.static_samplers.push(sampler);
        root_signature_desc.static_samplers.push(shadow_sampler);

        let root_signature =
            device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        // -- pipeline state object
        let pipeline_state_stream = SPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
        let pipeline_state = device
            .raw()
            .create_pipeline_state(&pipeline_state_stream_desc)?;

        let fovy: f32 = utils::PI / 4.0; // 45 degrees
        let znear = 0.1;

        // -- setup shadow mapping
        let shadow_mapping_pipeline = shadowmapping::setup_shadow_mapping_pipeline(
            &device, &mut direct_command_pool, Rc::downgrade(&dsv_heap), Rc::downgrade(&srv_heap), 128, 128)?;

        // -- setup imgui
        // ======================================================================
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
        let imgui_font_atlas_texture = fonts.build_rgba32_texture();
        let imgui_font_texture = texture_loader.create_texture_rgba32_from_bytes(
            imgui_font_atlas_texture.width,
            imgui_font_atlas_texture.height,
            imgui_font_atlas_texture.data,
        )?;
        drop(fonts);

        let orthomat_root_parameter = t12::SRootParameter {
            type_: t12::ERootParameterType::E32BitConstants,
            type_data: t12::ERootParameterTypeData::Constants {
                constants: t12::SRootConstants {
                    shader_register: 0,
                    register_space: 0,
                    num_32_bit_values: (size_of::<Mat4>() * 3 / 4) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let imgui_texture_root_parameter = {
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
                type_: t12::ERootParameterType::DescriptorTable,
                type_data: t12::ERootParameterTypeData::DescriptorTable {
                    table: root_descriptor_table,
                },
                shader_visibility: t12::EShaderVisibility::Pixel,
            }
        };

        let imgui_sampler = t12::SStaticSamplerDesc {
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

        let mut imgui_root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
        imgui_root_signature_desc.parameters.push(orthomat_root_parameter);
        let imgui_orthomat_root_param_idx = imgui_root_signature_desc.parameters.len() - 1;
        imgui_root_signature_desc.parameters.push(imgui_texture_root_parameter);
        let imgui_texture_descriptor_table_param_idx = imgui_root_signature_desc.parameters.len() - 1;
        imgui_root_signature_desc.static_samplers.push(imgui_sampler);

        let imgui_root_signature =
            device.create_root_signature(imgui_root_signature_desc, t12::ERootSignatureVersion::V1)?;

        // -- load shaders
        let imgui_vertblob = t12::read_file_to_blob("shaders_built/imgui_vertex.cso")?;
        let imgui_pixelblob = t12::read_file_to_blob("shaders_built/imgui_pixel.cso")?;

        let imgui_vert_byte_code = t12::SShaderBytecode::create(imgui_vertblob);
        let imgui_pixel_byte_code = t12::SShaderBytecode::create(imgui_pixelblob);

        let imgui_depth_stencil_desc = t12::SDepthStencilDesc {
            depth_enable: false,
            ..Default::default()
        };

        let mut imgui_input_layout_desc = t12::SInputLayoutDesc::create(&[
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
                "BLENDINDICES",
                0,
                t12::EDXGIFormat::R32UINT,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ]);


        let imgui_pipeline_state_stream = SImguiPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&imgui_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut imgui_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&imgui_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&imgui_pixel_byte_code),
            depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc::create(imgui_depth_stencil_desc),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let imgui_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&imgui_pipeline_state_stream);
        let imgui_pipeline_state = device
            .raw()
            .create_pipeline_state(&imgui_pipeline_state_stream_desc)?;

        let imgui_vert_buffer_resources = SMemVec::new(&SYSTEM_ALLOCATOR, 128, 0)?;
        let imgui_vert_buffer_views = SMemVec::new(&SYSTEM_ALLOCATOR, 128, 0)?;
        let imgui_index_buffer_resources = SMemVec::new(&SYSTEM_ALLOCATOR, 128, 0)?;
        let imgui_index_buffer_views = SMemVec::new(&SYSTEM_ALLOCATOR, 128, 0)?;


        // ======================================================================

        Ok(Self {
            factory,
            _adapter: adapter,
            device,

            direct_command_queue,
            direct_command_pool,
            _copy_command_queue: copy_command_queue,
            copy_command_pool,
            dsv_heap,
            srv_heap,

            _depth_texture_resource: None,
            _depth_texture_view: None,

            mesh_loader,
            texture_loader,

            scissorrect,
            fovy,
            znear,

            _vert_byte_code: vert_byte_code,
            _pixel_byte_code: pixel_byte_code,

            root_signature,
            pipeline_state,

            shadow_mapping_pipeline,

            imgui_font_texture,
            imgui_font_texture_id: imgui_ctxt.fonts().tex_id,
            imgui_root_signature,
            imgui_pipeline_state,
            imgui_orthomat_root_param_idx,
            imgui_texture_descriptor_table_param_idx,
            _imgui_vert_byte_code: imgui_vert_byte_code,
            _imgui_pixel_byte_code: imgui_pixel_byte_code,
            imgui_vert_buffer_resources,
            imgui_vert_buffer_views,
            imgui_index_buffer_resources,
            imgui_index_buffer_views,

            frame_fence_values: [0; 2],
        })
    }

    pub fn device(&self) -> &n12::SDevice {
        self.device.deref()
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn znear(&self) -> f32 {
        self.znear
    }

    pub fn create_window(
        &mut self,
        window_class: &safewindows::SWindowClass,
        title: &'static str,
        width: u32,
        height: u32,
    ) -> Result<n12::SD3D12Window, &'static str> {
        let window = n12::SD3D12Window::new(
            window_class,
            &self.factory,
            self.device.deref(),
            self.direct_command_queue.as_ref().borrow().deref(),
            title,
            width,
            height,
        )?;

        self.update_depth_texture_for_window(&window)?;

        Ok(window)
    }

    pub fn resize_window(&mut self, window: &mut n12::SD3D12Window, new_width: i32, new_height: i32) -> Result<(), &'static str> {
        window.resize(
            new_width as u32,
            new_height as u32,
            self.direct_command_queue.borrow_mut().deref_mut(),
            self.device.deref(),
        )?;

        self.update_depth_texture_for_window(window)?;

        Ok(())
    }

    pub fn new_model(&mut self, obj_file_path: &'static str, diffuse_weight: f32) -> Result<SModel, &'static str> {
        SModel::new_from_obj(obj_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight)
    }

    #[allow(dead_code)]
    pub fn ray_intersects(
        &self,
        model: &SModel,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {
        self.mesh_loader.ray_intersects(model, ray_origin, ray_dir, model_to_ray_space)
    }

    pub fn update_depth_texture_for_window(&mut self, window: &n12::SD3D12Window) -> Result<(), &'static str> {
        // -- depth texture
        #[allow(unused_variables)]
        let (mut _depth_texture_resource, mut _depth_texture_view) = n12::create_committed_depth_textures(
            window.width(),
            window.height(),
            1,
            &self.device,
            t12::EDXGIFormat::D32Float,
            t12::EResourceStates::DepthWrite,
            &mut self.direct_command_pool,
            &self.dsv_heap,
        )?;

        self._depth_texture_resource = Some(_depth_texture_resource);
        self._depth_texture_view = Some(_depth_texture_view);


        // -- $$$FRK(TODO): why do we do this?
        let maxframefencevalue =
            std::cmp::max(self.frame_fence_values[0], self.frame_fence_values[1]);
        self.frame_fence_values[0] = maxframefencevalue;
        self.frame_fence_values[1] = maxframefencevalue;

        Ok(())
    }

    pub fn render(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4, models: &[SModel], model_xforms: &[STransform]) -> Result<(), &'static str> {
        let viewport = t12::SViewport::new(
            0.0,
            0.0,
            window.width() as f32,
            window.height() as f32,
            None,
            None,
        );

        let perspective_matrix: Mat4 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh_zo(aspect, self.fovy(), self.znear(), zfar)
        };

        // -- wait for buffer to be available
        self.direct_command_queue.borrow()
            .wait_for_internal_fence_value(self.frame_fence_values[window.currentbackbufferindex()]);

        // -- render shadowmaps
        {
            let handle = self.direct_command_pool.alloc_list()?;
            let mut list = self.direct_command_pool.get_list(handle)?;

            self.shadow_mapping_pipeline.render(
                &self.mesh_loader,
                &Vec3::new(5.0, 5.0, 5.0),
                list.deref_mut(),
                models,
                model_xforms,
            )?;

            drop(list);

            let fence_val = self.direct_command_pool.execute_and_free_list(handle)?;
            self.direct_command_pool.wait_for_internal_fence_value(fence_val);
        }

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let handle = self.direct_command_pool.alloc_list()?;

            {
                let mut list = self.direct_command_pool.get_list(handle)?;

                let backbuffer = window.currentbackbuffer();
                let render_target_view = window.currentrendertargetdescriptor()?;
                let depth_texture_view = self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);

                // -- transition to render target
                list.transition_resource(
                    backbuffer,
                    t12::EResourceStates::Present,
                    t12::EResourceStates::RenderTarget,
                )?;

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];
                list.clear_render_target_view(
                    window.currentrendertargetdescriptor()?,
                    &clearcolour,
                )?;
                list.clear_depth_stencil_view(self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;

                // -- set up pipeline
                list.set_pipeline_state(&self.pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&self.root_signature.raw());

                // -- setup rasterizer state
                list.rs_set_viewports(&[&viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&self.scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

                self.srv_heap.with_raw_heap(|rh| {
                    list.set_descriptor_heaps(&[rh]);
                });

                let view_perspective = perspective_matrix * view_matrix;
                for modeli in 0..models.len() {
                    list.set_graphics_root_descriptor_table(3, &self.shadow_mapping_pipeline.srv().gpu_descriptor(0));
                    models[modeli].set_texture_root_parameters(&self.texture_loader, list.deref_mut(), 1, 2);
                    self.mesh_loader.render(models[modeli].mesh, list.deref_mut(), &view_perspective, &model_xforms[modeli])?;
                }
            }

            // -- execute on the queue
            assert_eq!(window.currentbackbufferindex(), backbufferidx);
            self.direct_command_pool.execute_and_free_list(handle)?;

            Ok(())
        }
    }

    pub fn setup_imgui_draw_data_resources(&mut self, draw_data: &imgui::DrawData) -> Result<(), &'static str> {
        for draw_list in draw_data.draw_lists() {
            let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
                //STACK_ALLOCATOR.with(|sa| {
                    //let vert_vec = SMemVec::<SImguiVertData>::new(draw_list.vtx_buffer().len(), 0, &sa)?;
                    //let idx_vec = SMemVec::<u16>::new(draw_list.idx_buffer().len(), 0, &sa)?;
                    //panic!("need to impl copy to vecs above.");

                    let handle = self.copy_command_pool.alloc_list()?;
                    let mut copy_command_list = self.copy_command_pool.get_list(handle)?;

                    // -- $$$FRK(TODO): we should be able to update the data in the resource, rather than creating a new one?
                    let mut vertbufferresource = {
                        let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                        copy_command_list.update_buffer_resource(
                            self.device.deref(),
                            draw_list.vtx_buffer(),
                            vertbufferflags
                        )?
                    };
                    let vertexbufferview = vertbufferresource
                        .destinationresource
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
                        .destinationresource
                        .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

                    drop(copy_command_list);

                    let fence_val = self.copy_command_pool.execute_and_free_list(handle)?;
                    // -- $$$FRK(TODO): we should be able to sychronize between this and the direct queue?
                    self.copy_command_pool.wait_for_internal_fence_value(fence_val);
                    self.copy_command_pool.free_allocators();

                    unsafe {
                        vertbufferresource.destinationresource.set_debug_name("imgui vert dest");
                        vertbufferresource.intermediateresource.set_debug_name("imgui vert inter");
                        indexbufferresource.destinationresource.set_debug_name("imgui index dest");
                        indexbufferresource.intermediateresource.set_debug_name("imgui index inter");
                    }

                    (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
                //})
            };

            // -- save the data until the next frame? double buffering will probably break this
            self.imgui_vert_buffer_resources.push(vertbufferresource.destinationresource);
            self.imgui_vert_buffer_views.push(vertexbufferview);
            self.imgui_index_buffer_resources.push(indexbufferresource.destinationresource);
            self.imgui_index_buffer_views.push(indexbufferview);
        }

        Ok(())
    }

    pub fn render_imgui(&mut self, window: &mut n12::SD3D12Window, draw_data: &imgui::DrawData) -> Result<(), &'static str> {
        let backbufferidx = window.currentbackbufferindex();

        let handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(handle)?;

        // -- set up pipeline
        list.set_pipeline_state(&self.imgui_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.imgui_root_signature.raw());

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

        self.srv_heap.with_raw_heap(|rh| {
            list.set_descriptor_heaps(&[rh]);
        });

        let ortho_matrix: Mat4 = {
            let znear = 0.0;
            let zfar = 1.0;

            let left = draw_data.display_pos[0];
            let right = draw_data.display_pos[0] + draw_data.display_size[0];
            let bottom = draw_data.display_pos[1] + draw_data.display_size[1];
            let top = draw_data.display_pos[1];

            glm::ortho_lh_zo(left, right, bottom, top, znear, zfar)
        };

        list.set_graphics_root_32_bit_constants(self.imgui_orthomat_root_param_idx as u32, &ortho_matrix, 0);

        for (i, draw_list) in draw_data.draw_lists().enumerate() {

            // -- set up input assembler
            list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
            list.ia_set_vertex_buffers(0, &[&self.imgui_vert_buffer_views[i]]);
            list.ia_set_index_buffer(&self.imgui_index_buffer_views[i]);

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
                            bottom: f32::min(clip_rect[2], window.height() as f32).floor() as i32,
                        };

                        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

                        let texture = self.get_imgui_texture(texture_id);
                        list.set_graphics_root_descriptor_table(
                            self.imgui_texture_descriptor_table_param_idx,
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
        self.direct_command_pool.execute_and_free_list(handle)?;

        Ok(())
    }

    pub fn present(&mut self, window: &mut n12::SD3D12Window) -> Result<(), &'static str> {
        let handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(handle)?;

        let backbuffer = window.currentbackbuffer();

        // -- transition to present
        list.transition_resource(
            backbuffer,
            t12::EResourceStates::RenderTarget,
            t12::EResourceStates::Present,
        )?;

        drop(list);
        self.direct_command_pool.execute_and_free_list(handle)?;

        self.frame_fence_values[window.currentbackbufferindex()] =
            self.direct_command_queue.borrow_mut().signal_internal_fence()?;

        // -- present the swap chain and switch to next buffer in swap chain
        window.present()?;

        Ok(())
    }

    #[allow(unused_variables)]
    fn get_imgui_texture(&self, texture_id: imgui::TextureId) -> SPoolHandle {
        if texture_id == self.imgui_font_texture_id {
            return self.imgui_font_texture;
        }

        panic!("We don't have any other textures!!!!");
    }

    pub fn flush(&mut self) -> Result<(), &'static str> {
        self.direct_command_queue.borrow_mut().flush_blocking()
    }
}
