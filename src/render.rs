// -- std includes
use std::cell::RefCell;
use std::mem::size_of;
use std::io::Write;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

// -- crate includes
use arrayvec::{ArrayVec};
use serde::{Serialize, Deserialize};
use glm::{Vec3, Mat4};

use niced3d12 as n12;
use typeyd3d12 as t12;
use allocate::{SMemVec, STACK_ALLOCATOR};
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

#[derive(Serialize, Deserialize)]
struct SBuiltShaderMetadata {
    src_write_time: std::time::SystemTime,
}

pub struct SRender {
    device: Rc<n12::SDevice>,

    direct_command_queue: Rc<RefCell<n12::SCommandQueue>>,
    direct_command_pool: n12::SCommandListPool,
    copy_command_queue: Rc<RefCell<n12::SCommandQueue>>,
    dsv_heap: Rc<n12::SDescriptorAllocator>,
    srv_heap: Rc<n12::SDescriptorAllocator>,

    mesh_loader: SMeshLoader,
    texture_loader: STextureLoader,

    scissorrect: t12::SRect,
    fovy: f32,
    znear: f32,

    _vert_byte_code: t12::SShaderBytecode,
    _pixel_byte_code: t12::SShaderBytecode,

    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,

    shadow_mapping_pipeline: SShadowMappingPipeline,
}

pub fn compile_shaders_if_changed() {
    let shaders = [
        ("pixel", "ps_6_0"),
        ("vertex", "vs_6_0"),
        ("shadow_pixel", "ps_6_0"),
        ("shadow_vertex", "vs_6_0"),
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

impl SRender{
    pub fn new(winapi: &rustywindows::SWinAPI) -> Result<Self, &'static str> {
        // -- initialize debug
        let debuginterface = t12::SDebugInterface::new()?;
        debuginterface.enabledebuglayer();

        let mut factory = n12::SFactory::create()?;
        let mut adapter = factory.create_best_adapter()?;
        let mut device = adapter.create_device()?;

        // -- set up command queues
        let direct_command_queue = Rc::new(RefCell::new(
            device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Direct)?,
        ));
        let mut direct_command_pool =
            n12::SCommandListPool::create(&device, Rc::downgrade(&direct_command_queue), &winapi.rawwinapi(), 1, 10)?;

        let dsv_heap = n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::DepthStencil,
            t12::SDescriptorHeapFlags::none(),
        )?;

        let srv_heap = n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess,
            t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::ShaderVisible),
        )?;

        let scissorrect = t12::SRect {
            left: 0,
            right: std::i32::MAX,
            top: 0,
            bottom: std::i32::MAX,
        };

        let copy_command_queue = Rc::new(RefCell::new(
            device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Copy)?,
        ));
        let mut mesh_loader = SMeshLoader::new(&device, &winapi, Rc::downgrade(&copy_command_queue), 23948934, 1024)?;
        let mut texture_loader = STextureLoader::new(&device, &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), &srv_heap, 9323, 1024)?;

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
            &device, &mut directcommandpool, &dsv_heap, &srv_heap, 128, 128)?;

        Ok(Self {
            direct_command_queue,
            direct_command_pool,
            copy_command_queue,
            dsv_heap,
            srv_heap,

            mesh_loader,
            texture_loader,

            scissorrect,
            fovy,
            znear,

            _vert_byte_code,
            _pixel_byte_code,

            root_signature,
            pipeline_state,

            shadow_mapping_pipeline,
        })
    }

    pub fn device(&self) -> &n12::SDevice {
        self.device
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn znear(&self) -> f32 {
        self.znear
    }

    pub fn create_window(
        &self,
        window_class: &safewindows::SWindowClass,
        title: &'static str,
        width: u32,
        height: u32,
    ) -> Result<n12::SD3D12Window, &'static str> {
        let mut window = n12::SD3D12Window::new(
            window_class,
            self.factory,
            self.device,
            self.direct_command_queue.as_ref().borrow().deref(),
            title,
            width,
            height,
        )?;

        self.update_depth_texture_for_window(window);

        window
    }

    pub fn resize_window(&mut self, window: &mut n12::SD3D12Window, new_width: i32, new_height: i32) {
        window.resize(
            newwidth as u32,
            newheight as u32,
            commandqueue.borrow_mut().deref_mut(),
            &device,
        )?;

        self.update_depth_texture_for_window(window);
    }

    pub fn new_model(&mut self, obj_file_path: &'static str, diffuse_weight: f32) -> Result<SModel, &'static str> {
        SModel::new_from_obj(obj_file_path, self.mesh_loader, self.texture_loader, diffuse_weight)
    }

    pub fn ray_intersects(
        &self,
        model: &SModel,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {
        mesh_loader.ray_intersects(model, ray_origin, ray_dir, model_to_ray_space)
    }

    pub fn update_depth_texture_for_window() {
        // -- depth texture
        #[allow(unused_variables)]
        let (mut _depth_texture_resource, mut _depth_texture_view) = n12::create_committed_depth_textures(
            width,
            height,
            1,
            &device,
            t12::EDXGIFormat::D32Float,
            t12::EResourceStates::DepthWrite,
            &mut directcommandpool,
            &dsv_heap,
        )?;

        self.depth_texture_resource = _depth_texture_resource;
        self.depth_texture_view = _depth_texture_view;


        // -- $$$FRK(TODO): why do we do this?
        let maxframefencevalue =
            std::cmp::max(self.framefencevalues[0], self.framefencevalues[1]);
        self.framefencevalues[0] = self.maxframefencevalue;
        self.framefencevalues[1] = self.maxframefencevalue;
    }

    pub fn render(&self, view_matrix: &Mat4, models: &[&SModel], model_xforms: &[&STransform]) {
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
            glm::perspective_lh_zo(aspect, fovy, znear, zfar)
        };

        // -- wait for buffer to be available
        commandqueue.borrow()
            .wait_for_internal_fence_value(framefencevalues[window.currentbackbufferindex()]);

        // -- render shadowmaps
        {
            let handle = directcommandpool.alloc_list()?;
            let list = directcommandpool.get_list(handle)?;

            shadow_mapping_pipeline.render(
                &mesh_loader,
                &Vec3::new(5.0, 5.0, 5.0),
                list,
                &models,
                &model_xforms,
            )?;

            let fence_val = directcommandpool.execute_and_free_list(handle)?;
            directcommandpool.wait_for_internal_fence_value(fence_val);
        }

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let handle = directcommandpool.alloc_list()?;

            {
                let list = directcommandpool.get_list(handle)?;

                let backbuffer = window.currentbackbuffer();
                let render_target_view = window.currentrendertargetdescriptor()?;
                let depth_texture_view = _depth_texture_view.cpu_descriptor(0);

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
                list.clear_depth_stencil_view(_depth_texture_view.cpu_descriptor(0), 1.0)?;

                // -- set up pipeline
                list.set_pipeline_state(&pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&root_signature.raw());

                // -- setup rasterizer state
                list.rs_set_viewports(&[&viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

                srv_heap.with_raw_heap(|rh| {
                    list.set_descriptor_heaps(&[rh]);
                });

                let view_perspective = perspective_matrix * view_matrix;
                for modeli in 0..models.len() {
                    list.set_graphics_root_descriptor_table(3, &shadow_mapping_pipeline.srv().gpu_descriptor(0));
                    models[modeli].set_texture_root_parameters(&texture_loader, list, 1, 2);
                    mesh_loader.render(models[modeli].mesh, list, &view_perspective, &model_xforms[modeli])?;
                }

                // -- transition to present
                list.transition_resource(
                    backbuffer,
                    t12::EResourceStates::RenderTarget,
                    t12::EResourceStates::Present,
                )?;
            }

            // -- execute on the queue
            assert_eq!(window.currentbackbufferindex(), backbufferidx);
            directcommandpool.execute_and_free_list(handle)?;
            framefencevalues[window.currentbackbufferindex()] =
                commandqueue.borrow_mut().signal_internal_fence()?;

            // -- present the swap chain and switch to next buffer in swap chain
            window.present()?;
        }
    }

    pub fn flush(&mut self) {
        commandqueue.borrow_mut().flush_blocking()?;
    }
}
