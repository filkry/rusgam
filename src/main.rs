extern crate arrayvec;
extern crate nalgebra_glm as glm;
extern crate tinytga;
extern crate tobj;
extern crate winapi;
extern crate wio;
extern crate bitflags;
extern crate serde_json;
extern crate serde;

//mod math;
#[macro_use]
mod safewindows;
mod allocate;
mod collections;
mod directxgraphicssamples;
mod niced3d12;
mod rustywindows;
mod typeyd3d12;
mod utils;
mod enumflags;
mod camera;
mod model;
mod shadowmapping;
mod level;

// -- std includes
use std::cell::RefCell;
use std::mem::size_of;
use std::io::Write;

// -- crate includes
use arrayvec::{ArrayVec};
use serde::{Serialize, Deserialize};
use glm::{Vec3, Mat4};

use niced3d12 as n12;
use typeyd3d12 as t12;
use allocate::{SMemVec, STACK_ALLOCATOR};
use utils::{STransform};
use model::{SModel, SMeshLoader, STextureLoader};

pub struct SInput {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    space: bool,
    c: bool,
    mouse_dx: i32,
    mouse_dy: i32,
}

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

fn compile_shaders_if_changed() {
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

fn main_d3d12() -> Result<(), &'static str> {
    compile_shaders_if_changed();

    // -- initialize debug
    let debuginterface = t12::SDebugInterface::new()?;
    debuginterface.enabledebuglayer();

    // -- setup window and command queue
    let winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut factory = n12::SFactory::create()?;
    let mut adapter = factory.create_best_adapter()?;
    let mut device = adapter.create_device()?;

    let commandqueue = RefCell::new(
        device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Direct)?,
    );
    let mut directcommandpool =
        n12::SCommandListPool::create(&device, &commandqueue, &winapi.rawwinapi(), 1, 10)?;

    let mut window = n12::SD3D12Window::new(
        &windowclass,
        &factory,
        &mut device,
        &mut commandqueue.borrow_mut(),
        "rusgam",
        800,
        600,
    )?;

    window.init_render_target_views(&mut device)?;
    window.show();

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

    let mut viewport = t12::SViewport::new(
        0.0,
        0.0,
        window.width() as f32,
        window.height() as f32,
        None,
        None,
    );
    let scissorrect = t12::SRect {
        left: 0,
        right: std::i32::MAX,
        top: 0,
        bottom: std::i32::MAX,
    };

    let copycommandqueue = RefCell::new(
        device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Copy)?,
    );
    let mut mesh_loader = SMeshLoader::new(&device, &winapi, &copycommandqueue, 23948934, 1024)?;
    let mut texture_loader = STextureLoader::new(&device, &winapi, &copycommandqueue, &commandqueue, &srv_heap, 9323, 1024)?;

    let model = SModel::new_from_obj("assets/first_test_asset.obj", &mut mesh_loader, &mut texture_loader, 1.0)?;
    let model2 = SModel::new_from_obj("assets/first_test_asset.obj", &mut mesh_loader, &mut texture_loader, 1.0)?;
    let model3 = SModel::new_from_obj("assets/test_untextured_flat_colour_cube.obj", &mut mesh_loader, &mut texture_loader, 1.0)?;

    let room_model = SModel::new_from_obj("assets/test_open_room.obj", &mut mesh_loader, &mut texture_loader, 1.0)?;

    let debug_model = SModel::new_from_obj("assets/debug_icosphere.obj", &mut mesh_loader, &mut texture_loader, 1.0)?;

    //let fixed_size_model = SModel::new_from_obj("assets/test_untextured_flat_colour_cube.obj", &device, &mut copycommandpool, &mut directcommandpool, &srv_heap, true, 1.0)?;

    let translation_widget = SModel::new_from_obj("assets/arrow_widget.obj", &mut mesh_loader, &mut texture_loader, 0.8)?;

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
                num_32_bit_values: 3,
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

    // -- depth texture
    #[allow(unused_variables)]
    let (mut _depth_texture_resource, mut _depth_texture_view) = n12::create_committed_depth_textures(
        window.width(),
        window.height(),
        1,
        &device,
        t12::EDXGIFormat::D32Float,
        t12::EResourceStates::DepthWrite,
        &mut directcommandpool,
        &dsv_heap,
    )?;

    // -- setup shadow mapping
    let shadow_mapping_pipeline = shadowmapping::setup_shadow_mapping_pipeline(
        &device, &mut directcommandpool, &dsv_heap, &srv_heap, 128, 128)?;

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    let mut shouldquit = false;

    let start_time = winapi.curtimemicroseconds();
    let rot_axis = Vec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = SInput{
        w: false,
        a: false,
        s: false,
        d: false,
        space: false,
        c: false,

        mouse_dx: 0,
        mouse_dy: 0,
    };

    let mut last_ray_hit_pos = Vec3::new(0.0, 0.0, 0.0);

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let total_time = curframetime - start_time;

        // -- update
        let cur_angle = ((total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        let model_xform = STransform::new_rotation(&glm::quat_angle_axis(cur_angle, &rot_axis));
        let model2_xform = STransform::new_translation(&glm::Vec3::new(3.0, 0.0, 0.0));
        let model3_xform = STransform::new_translation(&glm::Vec3::new(0.0, 2.0, 0.0));
        let room_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, -2.0, 0.0));

        let debug_model_xform = STransform::new_translation(&last_ray_hit_pos);

        let mut fixed_size_model_xform = STransform::new_translation(&glm::Vec3::new(0.0, 5.0, 0.0));

        let fovy: f32 = utils::PI / 4.0; // 45 degrees
        let znear = 0.1;


        {
            let fovx = utils::fovx(fovy, window.width(), window.height());

            let to_fixed = fixed_size_model_xform.t - camera.pos_world;
            let dist = glm::length(&to_fixed);

            let angle_from_forward = glm::angle(&to_fixed, &camera.forward_world());
            let proj_dist = znear / (angle_from_forward).cos();

            // -- the whole idea of this code is to build a ratio of the similar
            // -- triangle from the object in world space to the amount of space
            // -- 1 unit will take up on the near plane projection, then scale it
            // -- so that space is constant
            let proj_ratio = proj_dist / dist;

            let unit_in_proj_space = 1.0 * proj_ratio;

            let total_proj_space = 2.0 * znear * (fovx / 2.0).tan();
            let desired_proj_space = total_proj_space / 10.0;

            let scale = desired_proj_space / unit_in_proj_space;

            fixed_size_model_xform.s = scale;
        }

        let perspective_matrix: Mat4 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh_zo(aspect, fovy, znear, zfar)
        };

        camera.update_from_input(&input, dts);
        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.world_to_view_matrix();

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // -- wait for buffer to be available
        commandqueue
            .borrow()
            .wait_for_internal_fence_value(framefencevalues[window.currentbackbufferindex()]);

        let models = [
            &model,
            &model2,
            &model3,
            &room_model,
            &debug_model,
            //&fixed_size_model,
        ];
        let model_xforms = [
            &model_xform,
            &model2_xform,
            &model3_xform,
            &room_model_xform,
            &debug_model_xform,
            //&fixed_size_model_xform,
        ];

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
                    mesh_loader.render(models[modeli].mesh, list, &view_perspective, &model_xforms[modeli]);
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

        lastframetime = curframetime;
        _framecount += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            let msg = window.pollmessage();
            match msg {
                None => break,
                Some(m) => match m {
                    safewindows::EMsgType::Paint => {
                        //println!("Paint!");
                        window.dummyrepaint();
                    }
                    safewindows::EMsgType::KeyDown { key } => match key {
                        safewindows::EKey::Q => {
                            shouldquit = true;
                            //println!("Q keydown");
                        }
                        safewindows::EKey::W => input.w = true,
                        safewindows::EKey::A => input.a = true,
                        safewindows::EKey::S => input.s = true,
                        safewindows::EKey::D => input.d = true,
                        safewindows::EKey::Space => input.space = true,
                        safewindows::EKey::C => input.c = true,
                        _ => (),
                    },
                    safewindows::EMsgType::KeyUp { key } => match key {
                        safewindows::EKey::W => input.w = false,
                        safewindows::EKey::A => input.a = false,
                        safewindows::EKey::S => input.s = false,
                        safewindows::EKey::D => input.d = false,
                        safewindows::EKey::Space => input.space = false,
                        safewindows::EKey::C => input.c = false,
                        _ => (),
                    },
                    safewindows::EMsgType::LButtonDown{ x_pos, y_pos } => {
                        println!("Left button down: {}, {}", x_pos, y_pos);

                        let half_camera_near_clip_height = (fovy/2.0).tan() * znear;
                        let half_camera_near_clip_width = ((window.width() as f32) / (window.height() as f32)) * half_camera_near_clip_height;

                        let near_clip_top_left_camera_space = Vec3::new(-half_camera_near_clip_width, half_camera_near_clip_height, znear);
                        let near_clip_deltax_camera_space = Vec3::new(2.0 * half_camera_near_clip_width, 0.0, 0.0);
                        let near_clip_deltay_camera_space = Vec3::new(0.0, -2.0 * half_camera_near_clip_height, 0.0);

                        let pct_width = (x_pos as f32) / (window.width() as f32);
                        let pct_height = (y_pos as f32) / (window.height() as f32);

                        let to_z_near_camera_space = near_clip_top_left_camera_space +
                            pct_width * near_clip_deltax_camera_space +
                            pct_height * near_clip_deltay_camera_space;

                        println!("to_z_near_camera_space: {:?}", to_z_near_camera_space);

                        let world_to_view = camera.world_to_view_matrix();
                        let view_to_world = glm::inverse(&world_to_view);

                        let to_z_near_world_space = view_to_world * utils::vec3_to_homogenous(&to_z_near_camera_space, 0.0);

                        let mut min_t = std::f32::MAX;
                        let mut min_model_i = None;
                        let mut min_pos = Vec3::new(0.0, 0.0, 0.0);

                        for modeli in 0..models.len() {
                            if let Some(t) = mesh_loader.ray_intersects(&model, &camera.pos_world, &to_z_near_world_space.xyz(), model_xforms[modeli]) {
                                if t < min_t {
                                    min_t = t;
                                    min_model_i = Some(modeli);
                                    min_pos = camera.pos_world + t * to_z_near_world_space.xyz();
                                }
                            }
                        }

                        if let Some(modeli) = min_model_i {
                            println!("Hit model {}", modeli);
                            last_ray_hit_pos = min_pos;
                        }
                    },
                    safewindows::EMsgType::Input{ raw_input } => {
                        if let safewindows::rawinput::ERawInputData::Mouse{data} = raw_input.data {
                            //println!("Frame {}: Raw Mouse: {}, {}", _framecount, data.last_x, data.last_y);
                            input.mouse_dx = data.last_x;
                            input.mouse_dy = data.last_y;
                        }
                    },
                    safewindows::EMsgType::Size => {
                        //println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        window.resize(
                            newwidth as u32,
                            newheight as u32,
                            &mut commandqueue.borrow_mut(),
                            &device,
                        )?;

                        let (new_resource, new_view) = n12::create_committed_depth_textures(
                            window.width(),
                            window.height(),
                            1,
                            &device,
                            t12::EDXGIFormat::D32Float,
                            t12::EResourceStates::DepthWrite,
                            &mut directcommandpool,
                            &dsv_heap,
                        )?;
                        _depth_texture_resource = new_resource;
                        _depth_texture_view = new_view;

                        viewport = t12::SViewport::new(
                            0.0,
                            0.0,
                            window.width() as f32,
                            window.height() as f32,
                            None,
                            None,
                        );

                        // -- $$$FRK(TODO): why do we do this?
                        let maxframefencevalue =
                            std::cmp::max(framefencevalues[0], framefencevalues[1]);
                        framefencevalues[0] = maxframefencevalue;
                        framefencevalues[1] = maxframefencevalue;
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }

        // -- increase frame time for testing
        //std::thread::sleep(std::time::Duration::from_millis(111));
    }

    // -- wait for all commands to clear
    commandqueue.borrow_mut().flush_blocking()?;

    Ok(())
}

fn debug_test() {}

fn main() {
    debug_test();

    main_d3d12().unwrap();
}
