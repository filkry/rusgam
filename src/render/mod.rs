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
use model;
use model::{SModel, SMeshLoader, STextureLoader};
use safewindows;
use rustywindows;
use utils;
use utils::{STransform};

mod shadowmapping;
mod render_imgui;

use self::render_imgui::{SRenderImgui};

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
struct SNoDepthPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[allow(unused_variables)]
#[allow(unused_mut)]
#[repr(C)]
struct SDebugLinePipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

#[derive(Serialize, Deserialize)]
struct SBuiltShaderMetadata {
    src_write_time: std::time::SystemTime,
}

#[allow(dead_code)]
struct SDebugLine {
    start: Vec3,
    end: Vec3,
    colour: Vec3,
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

    no_depth_pipeline_state: t12::SPipelineState,

    // -- imgui stuff
    render_imgui: SRenderImgui<'a>,

    // -- debug render stuff
    debug_line_pipeline_state: t12::SPipelineState,
    debug_line_root_signature: n12::SRootSignature,
    debug_line_vp_root_param_idx: usize,
    _debug_line_vert_byte_code: t12::SShaderBytecode,
    _debug_line_pixel_byte_code: t12::SShaderBytecode,
    debug_lines: SMemVec::<'a, SDebugLine>,
    debug_line_vertex_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    debug_line_vertex_buffer_resource: [Option<n12::SResource>; 2],
    debug_line_vertex_buffer_view: [Option<t12::SVertexBufferView>; 2],

    frame_fence_values: [u64; 2],

    // -- these things need to drop last, due to Weak references to them in the above structs
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
        ("debug_line_vertex", "vs_6_0"),
        ("debug_line_pixel", "ps_6_0"),
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
                       .arg("-Od")
                       .arg("-Zi")
                       .arg("-Qembed_debug")
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
        let mesh_loader = SMeshLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), 23948934, 1024)?;
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

        let render_imgui = SRenderImgui::new(imgui_ctxt, &mut texture_loader, &device)?;

        // ======================================================================
        // -- setup no depth test pipeline
        // ======================================================================
        let no_depth_depth_stencil_desc = t12::SDepthStencilDesc {
            depth_enable: true,
            ..Default::default()
        };

        let no_depth_pipeline_state_stream = SNoDepthPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
            depth_stencil_desc: n12::SPipelineStateStreamDepthStencilDesc::create(no_depth_depth_stencil_desc),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let no_depth_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&no_depth_pipeline_state_stream);
        let no_depth_pipeline_state = device
            .raw()
            .create_pipeline_state(&no_depth_pipeline_state_stream_desc)?;

        // ======================================================================
        // -- setup debug line pipeline
        // ======================================================================
        let debug_line_root_signature_flags = {
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
                    num_32_bit_values: (size_of::<Mat4>() * 3 / 4) as u32,
                },
            },
            shader_visibility: t12::EShaderVisibility::Vertex,
        };

        let mut debug_line_root_signature_desc = t12::SRootSignatureDesc::new(debug_line_root_signature_flags);
        debug_line_root_signature_desc.parameters.push(vp_root_parameter);
        let debug_line_vp_root_param_idx = debug_line_root_signature_desc.parameters.len() - 1;

        let debug_line_root_signature =
            device.create_root_signature(debug_line_root_signature_desc,
                                         t12::ERootSignatureVersion::V1)?;

        let mut debug_line_input_layout_desc = t12::SInputLayoutDesc::create(&[
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

        let debug_line_vertblob = t12::read_file_to_blob("shaders_built/debug_line_vertex.cso")?;
        let debug_line_pixelblob = t12::read_file_to_blob("shaders_built/debug_line_pixel.cso")?;

        let debug_line_vert_byte_code = t12::SShaderBytecode::create(debug_line_vertblob);
        let debug_line_pixel_byte_code = t12::SShaderBytecode::create(debug_line_pixelblob);

        let debug_line_pipeline_state_stream = SDebugLinePipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&debug_line_root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut debug_line_input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Line,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(&debug_line_vert_byte_code),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(&debug_line_pixel_byte_code),
            depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
                t12::EDXGIFormat::D32Float,
            ),
            rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
        };
        let debug_line_pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&debug_line_pipeline_state_stream);
        let debug_line_pipeline_state = device
            .raw()
            .create_pipeline_state(&debug_line_pipeline_state_stream_desc)?;

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

            no_depth_pipeline_state,

            render_imgui,

            debug_line_pipeline_state,
            debug_line_root_signature,
            debug_line_vp_root_param_idx,
            _debug_line_vert_byte_code: debug_line_vert_byte_code,
            _debug_line_pixel_byte_code: debug_line_pixel_byte_code,
            debug_lines: SMemVec::new(&SYSTEM_ALLOCATOR, 1024, 0)?,
            debug_line_vertex_buffer_intermediate_resource: [None, None],
            debug_line_vertex_buffer_resource: [None, None],
            debug_line_vertex_buffer_view: [None, None],

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

    pub fn new_model(&mut self, obj_file_path: &'static str, diffuse_weight: f32, is_lit: bool) -> Result<SModel, &'static str> {
        SModel::new_from_obj(obj_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight, is_lit)
    }

    #[allow(dead_code)]
    pub fn ray_intersects(
        &self,
        model: &SModel,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {
        if model.pickable == false {
            return None;
        }

        self.mesh_loader.ray_intersects(model.mesh, ray_origin, ray_dir, model_to_ray_space)
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

        // -- reminder: D3D clip space is (-1, 1) x, (-1, 1) y, (0, 1) znear-zfar
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
                // -- $$$FRK(TODO): could make a model where you call beginrender() to get a render state that will transition the resource on create and drop
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

    pub fn render_draw_over(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4, models: &[SModel], model_xforms: &[STransform]) -> Result<(), &'static str> {
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

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let handle = self.direct_command_pool.alloc_list()?;

            {
                let mut list = self.direct_command_pool.get_list(handle)?;

                // -- clear depth texture so we can draw over
                list.clear_depth_stencil_view(self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;

                let render_target_view = window.currentrendertargetdescriptor()?;
                let depth_texture_view = self._depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);

                // -- set up pipeline
                list.set_pipeline_state(&self.no_depth_pipeline_state);
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

    pub fn render_debug_lines(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4) -> Result<(), &'static str> {
        let back_buffer_idx = window.currentbackbufferindex();

        /* A very basic test
        self.debug_lines.push(SDebugLine{
            start: Vec3::new(-5.0, 2.0, 0.0),
            end: Vec3::new(5.0, 2.0, 0.0),
            colour: Vec3::new(1.0, 0.0, 0.0),
        });
        */

        if self.debug_lines.len() == 0 {
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
        STACK_ALLOCATOR.with(|sa| -> Result<(), &'static str> {
            let mut vertex_buffer_data = SMemVec::new(
                sa,
                self.debug_lines.len() * 2,
                0,
            )?;

            for line in self.debug_lines.as_slice() {
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

            self.debug_line_vertex_buffer_intermediate_resource[back_buffer_idx] =
                Some(vert_buffer_resource.intermediateresource);
            self.debug_line_vertex_buffer_resource[back_buffer_idx] =
                Some(vert_buffer_resource.destinationresource);
            self.debug_line_vertex_buffer_view[back_buffer_idx] = Some(vertex_buffer_view);

            Ok(())
        })?;

        // -- set up pipeline and render lines
        let handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(handle)?;

        list.set_pipeline_state(&self.debug_line_pipeline_state);
        // root signature has to be set explicitly despite being on PSO, according to tutorial
        list.set_graphics_root_signature(&self.debug_line_root_signature.raw());

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

        list.set_graphics_root_32_bit_constants(self.debug_line_vp_root_param_idx as u32,
                                                &view_perspective, 0);

        // -- set up input assembler
        list.ia_set_primitive_topology(t12::EPrimitiveTopology::LineList);
        let vert_buffer_view = self.debug_line_vertex_buffer_view[back_buffer_idx].
            as_ref().expect("should have generated resource earlier in this function");
        list.ia_set_vertex_buffers(0, &[vert_buffer_view]);

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };
        list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

        for i in 0..self.debug_lines.len() {
            list.draw_instanced(2, 1, (i * 2) as u32, 0);
        }

        // -- execute on the queue
        drop(list);
        assert_eq!(window.currentbackbufferindex(), back_buffer_idx);
        self.direct_command_pool.execute_and_free_list(handle)?;

        self.debug_lines.clear();

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

    pub fn add_debug_line(&mut self, start: &Vec3, end: &Vec3, color: &Vec3) {
        self.debug_lines.push(SDebugLine {
            start: start.clone(),
            end: end.clone(),
            colour: color.clone(),
        });
    }

    pub fn flush(&mut self) -> Result<(), &'static str> {
        self.direct_command_queue.borrow_mut().flush_blocking()
    }
}
