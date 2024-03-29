// -- std includes
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

// -- crate includes
use arrayvec::{ArrayVec};
use crate::camera;
use ::serde::{Serialize, Deserialize};
use crate::math::{Vec3, Mat4};

use crate::niced3d12 as n12;
use crate::typeyd3d12 as t12;
use crate::allocate::{STACK_ALLOCATOR};
use crate::collections::{SVec};
use crate::databucket::{SDataBucket};
use crate::entity::{SEntityBucket, SEntityHandle};
use crate::entity_animation;
use crate::entity_model;
use crate::game_context::{SGameContext, SFrameContext};
use crate::model::{SMeshInstanceLoader, SMeshInstanceHandle, SMeshLoader, STextureLoader};
use crate::safewindows;
use crate::rustywindows;
use crate::utils;
use crate::utils::{STransform, SRay};

mod compute_skinning_pipeline;
mod shadowmapping;
mod render_imgui;
pub mod temp;
pub mod shaderbindings;

use self::render_imgui::{SRenderImgui};
use self::temp::{SRenderTemp};

#[derive(Serialize, Deserialize)]
struct SBuiltShaderMetadata {
    src_write_time: std::time::SystemTime,
}

#[allow(dead_code)]
pub struct SRenderContext {
    // -- these are all for the main view camera
    current_back_buffer_index: usize,
    render_target_view: t12::SCPUDescriptorHandle,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    view_projection_matrix: Mat4,
    viewport: t12::SViewport,
}

pub struct SRender {
    factory: n12::SFactory,
    _adapter: n12::SAdapter, // -- maybe don't need to keep
    device: Rc<n12::SDevice>,

    direct_command_pool: n12::SCommandListPool,
    copy_command_pool: n12::SCommandListPool,
    _copy_command_queue: Rc<RefCell<n12::SCommandQueue>>,
    direct_command_queue: Rc<RefCell<n12::SCommandQueue>>,

    dsv_heap: Rc<n12::SDescriptorAllocator>,
    cbv_srv_uav_heap: Rc<n12::SDescriptorAllocator>,

    mesh_loader: SMeshLoader,
    texture_loader: STextureLoader,
    mesh_instance_loader: SMeshInstanceLoader,

    // -- global RTV properties -----------------------------------
    _depth_texture_resource: Option<n12::SResource>,
    depth_texture_view: Option<n12::SDescriptorAllocatorAllocation>,

    scissorrect: t12::SRect,
    fovy: f32,
    znear: f32,

    frame_fence_values: [u64; 2],

    // -- pipelines -------------------------------------------

    // -- main world rendering pipeline
    vertex_hlsl: shaderbindings::SVertexHLSL,
    vertex_hlsl_bind: shaderbindings::SVertexHLSLBind,
    pixel_hlsl: shaderbindings::SPixelHLSL,
    pixel_hlsl_bind: shaderbindings::SPixelHLSLBind,

    root_signature: n12::SRootSignature,
    pipeline_state: t12::SPipelineState,

    // -- compute pipelines
    compute_skinning_pipeline: compute_skinning_pipeline::SComputeSkinningPipeline,

    // -- other rendering pipelines
    render_shadow_map: shadowmapping::SShadowMappingPipeline,
    render_imgui: SRenderImgui,
    render_temp: SRenderTemp,
}

pub fn compile_shaders_if_changed(d3d_debug: bool) {
    let shaders = [
        ("vertex", "vs_6_0"),
        ("pixel", "ps_6_0"),
        ("clip_space_only_vertex", "vs_6_0"),
        ("depth_only_pixel", "ps_6_0"),
        ("imgui_vertex", "vs_6_0"),
        ("imgui_pixel", "ps_6_0"),
        ("point_vertex", "vs_6_0"),
        ("point_pixel", "ps_6_0"),
        ("debug_line_vertex", "vs_6_0"),
        ("debug_line_pixel", "ps_6_0"),
        ("temp_mesh_vertex", "vs_6_0"),
        ("temp_mesh_pixel", "ps_6_0"),
        ("instance_mesh_vertex", "vs_6_0"),
        ("instance_mesh_pixel", "ps_6_0"),
        ("compute_skinning", "cs_6_0"),
    ];

    for (shader_name, type_) in &shaders {
        let mut needs_build = false;

        STACK_ALLOCATOR.with(|sa| {
            let mut shader_src_path_string = SVec::<u8>::new(&sa.as_ref(), 256, 0).unwrap();
            write!(&mut shader_src_path_string, "shaders/{}.hlsl", shader_name).unwrap();
            let shader_src_path = std::path::Path::new(shader_src_path_string.as_str());

            let mut built_shader_path_string = SVec::<u8>::new(&sa.as_ref(), 256, 0).unwrap();
            write!(&mut built_shader_path_string, "shaders_built/{}.cso", shader_name).unwrap();
            let built_shader_path = std::path::Path::new(built_shader_path_string.as_str());

            let mut build_metadata_path_string = SVec::<u8>::new(&sa.as_ref(), 256, 0).unwrap();
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

                // -- $$$FRK(FUTURE WORK): can I read into a custom buffer to not allocate from system heap?
                let build_shader_metadata_json_str = std::fs::read_to_string(build_metadata_path).unwrap();
                let build_shader_metadata : SBuiltShaderMetadata = serde_json::from_str(build_shader_metadata_json_str.as_str()).unwrap();

                if src_write_time != build_shader_metadata.src_write_time {
                    needs_build = true;
                }
                */
            }

            if needs_build {
                println!("Compiling shader {}...", shader_name);

                let mut command = std::process::Command::new("externals/dxc_2020_10-22/bin/x64/dxc.exe");

                command.arg("-E").arg("main") // -- entry point
                       .arg("-T").arg(type_) // -- type
                       .arg(shader_src_path)
                       .arg("-WX") // -- treat warnings as errors
                       .arg("-Fo").arg(built_shader_path); // -- output object file

                if d3d_debug {
                    command
                       .arg("-Od") // -- optimization level debug
                       .arg("-Zi") // -- enable debug information
                       .arg("-Qembed_debug"); // -- embed PDB in shader container (must be used with Zi)
                }

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

impl SRender {
    pub fn new(winapi: &rustywindows::SWinAPI, imgui_ctxt: &mut imgui::Context, d3d_debug: bool) -> Result<Self, &'static str> {
        // -- initialize debug
        if d3d_debug {
            let debuginterface = t12::SDebugInterface::new()?;
            debuginterface.enabledebuglayer();
        }

        let mut factory = n12::SFactory::create()?;
        let mut adapter = factory.create_best_adapter()?;
        let device = Rc::new(adapter.create_device(d3d_debug)?);

        // -- set up command queues
        let direct_command_queue = Rc::new(RefCell::new(
            device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Direct)?,
        ));
        unsafe { direct_command_queue.borrow_mut().set_debug_name("render direct queue"); }
        let mut direct_command_pool =
            n12::SCommandListPool::create(&device, Rc::downgrade(&direct_command_queue), &winapi.rawwinapi(), 1, 20)?;

        let dsv_heap = Rc::new(n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::DepthStencil,
            t12::SDescriptorHeapFlags::none(),
        )?);

        let cbv_srv_uav_heap = Rc::new(n12::descriptorallocator::SDescriptorAllocator::new(
            &device,
            32,
            t12::EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess,
            t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::ShaderVisible),
        )?);

        cbv_srv_uav_heap.with_raw_heap(|rh| {
            println!("cbv_srv_uav_heap gpu start: {:?}", rh.gpu_handle_heap_start());
        });

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
        let mut mesh_loader = SMeshLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), Rc::downgrade(&cbv_srv_uav_heap), 1024)?;
        let mut texture_loader = STextureLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), Rc::downgrade(&cbv_srv_uav_heap), 1024)?;

        // -- load shaders
        let vertex_hlsl = shaderbindings::SVertexHLSL::new()?;
        let pixel_hlsl = shaderbindings::SPixelHLSL::new()?;

        // -- root signature stuff
        let input_layout_desc = shaderbindings::SVertexHLSL::input_layout_desc();

        let root_signature_flags = t12::SRootSignatureFlags::create(&[
            t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
            t12::ERootSignatureFlags::DenyHullShaderRootAccess,
            t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
            t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        ]);

        let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);

        let vertex_hlsl_bind = vertex_hlsl.bind(&mut root_signature_desc);
        let pixel_hlsl_bind = pixel_hlsl.bind(&mut root_signature_desc);

        let root_signature =
            device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

        let mut rtv_formats = t12::SRTFormatArray {
            rt_formats: ArrayVec::new(),
        };
        rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

        // -- pipeline state object
        let mut pipeline_state_desc = {
            let mut temp = t12::SGraphicsPipeLineStateDesc::new_min(
                root_signature.raw().clone(),
                input_layout_desc,
                t12::EPrimitiveTopologyType::Triangle,
            );
            temp.vertex_shader = Some(vertex_hlsl.bytecode());
            temp.pixel_shader = Some(pixel_hlsl.bytecode());
            temp.depth_stencil_format = Some(t12::EDXGIFormat::D32Float);
            temp.rtv_formats = Some(rtv_formats);
            temp
        };
        let pipeline_state = device
            .raw()
            .create_graphics_pipeline_state(&mut pipeline_state_desc)?;

        let fovy: f32 = utils::PI / 4.0; // 45 degrees
        let znear = 0.1;

        let compute_skinning_pipeline = compute_skinning_pipeline::setup_pipeline(&device)?;

        let render_shadow_map = shadowmapping::setup_shadow_mapping_pipeline(
            &device, &mut direct_command_pool, Rc::downgrade(&dsv_heap), Rc::downgrade(&cbv_srv_uav_heap), 128, 128)?;
        let render_imgui = SRenderImgui::new(imgui_ctxt, &mut texture_loader, &device)?;
        let render_temp = SRenderTemp::new(&device, &mut mesh_loader, &mut texture_loader)?;

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
            cbv_srv_uav_heap,

            _depth_texture_resource: None,
            depth_texture_view: None,

            mesh_loader,
            texture_loader,

            scissorrect,
            fovy,
            znear,

            vertex_hlsl,
            vertex_hlsl_bind,
            pixel_hlsl,
            pixel_hlsl_bind,

            root_signature,
            pipeline_state,

            compute_skinning_pipeline,

            render_shadow_map,
            render_imgui,
            render_temp,

            frame_fence_values: [0; 2],
        })
    }

    pub fn shutdown(&mut self) {
        self.render_shadow_map.shutdown();
        self.mesh_loader.shutdown();
        self.texture_loader.shutdown();
        self.depth_texture_view = None;
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

    pub fn temp(&mut self) -> &mut SRenderTemp {
        &mut self.render_temp
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

    pub fn new_mesh_instance_from_obj(&mut self, obj_file_path: &'static str, diffuse_weight: f32, is_lit: bool) -> Result<SMeshInstanceHandle, &'static str> {
        self.mesh_instance_loader.new_from_obj(obj_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight, is_lit)
    }

    pub fn new_mesh_instance_from_gltf(&mut self, gltf_file_path: &'static str, diffuse_weight: f32, is_lit: bool) -> Result<SMeshInstanceHandle, &'static str> {
        self.mesh_instance_loader.new_from_gtlf.new_from_gltf(gltf_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight, is_lit)
    }

    pub fn mesh_loader(&self) -> &SMeshLoader {
        &self.mesh_loader
    }

    pub fn mesh_instance_loader(&self) -> &SMeshInstanceLoader {
        &self.mesh_instance_loader
    }

    pub fn mesh_instance_loader_mut(&mut self) -> &mut SMeshInstanceLoader {
        &mut self.mesh_instance_loader
    }

    #[allow(dead_code)]
    pub fn ray_intersects(
        &self,
        model: &SMeshInstanceHandle,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {
        if model.pickable == false {
            return None;
        }

        self.mesh_instance_loader.ray_intersects(model, ray_origin, ray_dir, model_to_ray_space)
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
        self.depth_texture_view = Some(_depth_texture_view);


        // -- $$$FRK(TODO): why do we do this?
        let maxframefencevalue =
            std::cmp::max(self.frame_fence_values[0], self.frame_fence_values[1]);
        self.frame_fence_values[0] = maxframefencevalue;
        self.frame_fence_values[1] = maxframefencevalue;

        Ok(())
    }

    pub fn render_frame(
        &mut self,
        window: &n12::SD3D12Window,
        view_matrix: &Mat4,
        entities: &mut SEntityBucket,
        entity_animation: &mut entity_animation::SBucket,
        entity_model: &entity_model::SBucket,
        imgui_draw_data: Option<&imgui::DrawData>,
    ) -> Result<(), &'static str> {
        let back_buffer_idx = window.currentbackbufferindex();
        assert!(back_buffer_idx == window.swapchain.current_backbuffer_index());

        // -- wait for buffer to be available
        self.direct_command_queue.borrow()
            .wait_for_internal_fence_value(self.frame_fence_values[back_buffer_idx]);

        // -- clear RTV and depth buffer, transition
        {
            let backbuffer = window.currentbackbuffer();

            let mut handle = self.direct_command_pool.alloc_list()?;
            let mut list = self.direct_command_pool.get_list(&handle)?;

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
            list.clear_depth_stencil_view(self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;

            drop(list);
            self.direct_command_pool.execute_and_free_list(&mut handle)?;
        }

        // -- kick off imgui copies
        if let Some(idd) = imgui_draw_data {
            self.setup_imgui_draw_data_resources(window, idd)?;
        }

        // -- reminder: D3D clip space is (-1, 1) x, (-1, 1) y, (0, 1) znear-zfar
        let perspective_matrix: Mat4 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            Mat4::new_perspective(aspect, self.fovy(), self.znear(), zfar)
        };

        let context = SRenderContext{
            current_back_buffer_index: window.currentbackbufferindex(),
            view_matrix: view_matrix.clone(),
            projection_matrix: perspective_matrix,
            view_projection_matrix: perspective_matrix * view_matrix,
            viewport: t12::SViewport::new(
                0.0,
                0.0,
                window.width() as f32,
                window.height() as f32,
                None,
                None,
            ),
            render_target_view: window.currentrendertargetdescriptor()?,
        };

        // -- update skinned buffers
        self.compute_skinning(entity_animation, entity_model)?;

        // -- $$$FRK(TODO): should initialize the shadow map depth buffer to empty, so we still get light if we don't render maps
        self.render_shadow_maps(entities, entity_model)?;
        self.render_world(&context, entities, entity_animation, entity_model)?;
        self.render_temp_in_world(&context)?;

        // -- clear depth buffer again
        {
            let mut handle = self.direct_command_pool.alloc_list()?;
            let mut list = self.direct_command_pool.get_list(&handle)?;
            list.clear_depth_stencil_view(self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;
            drop(list);
            self.direct_command_pool.execute_and_free_list(&mut handle)?;
        }

        self.render_temp_over_world(&context)?;
        if let Some(idd) = imgui_draw_data {
            self.render_imgui(window, idd)?;
        }

        self.render_temp.clear_tables_without_tokens();

        assert_eq!(window.currentbackbufferindex(), back_buffer_idx);

        Ok(())
    }

    fn compute_skinning(&mut self, entity_animation: &mut entity_animation::SBucket, entity_model: &entity_model::SBucket) -> Result<(), &'static str> {
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        self.compute_skinning_pipeline.compute(list.deref_mut(), &self.mesh_loader, entity_animation, entity_model);

        drop(list);

        let fence_val = self.direct_command_pool.execute_and_free_list(&mut handle)?;
        self.direct_command_pool.wait_for_internal_fence_value(fence_val); // $$$FRK(TODO) reconsider this fence

        Ok(())
    }

    pub fn render_shadow_maps(
        &mut self,
        entities: &SEntityBucket,
        entity_model: &entity_model::SBucket,
    ) -> Result<(), &'static str> {
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        self.render_shadow_map.render(
            &self.mesh_loader,
            &Vec3::new(5.0, 5.0, 5.0),
            list.deref_mut(),
            entities,
            entity_model,
        )?;

        drop(list);

        let fence_val = self.direct_command_pool.execute_and_free_list(&mut handle)?;
        self.direct_command_pool.wait_for_internal_fence_value(fence_val); // $$$FRK(TODO) reconsider this fence

        Ok(())
    }

    pub fn render_world(
        &mut self,
        context: &SRenderContext,
        entities: &SEntityBucket,
        entity_animation: &entity_animation::SBucket,
        entity_model: &entity_model::SBucket,
    ) -> Result<(), &'static str> {
        // -- render
        {
            let mut handle = self.direct_command_pool.alloc_list()?;

            {
                let mut list = self.direct_command_pool.get_list(&handle)?;

                let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);

                // -- set up pipeline
                list.set_pipeline_state(&self.pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&self.root_signature.raw());

                // -- setup rasterizer state
                list.rs_set_viewports(&[&context.viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&self.scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&context.render_target_view], false, &depth_texture_view);

                self.cbv_srv_uav_heap.with_raw_heap(|rh| {
                    list.set_descriptor_heaps(&[rh]);
                });

                self.vertex_hlsl.set_roots(
                    &self.vertex_hlsl_bind,
                    &mut list,
                    context.view_projection_matrix,
                    self.mesh_instance_loader.instance_data_buffer(),
                    self.mesh_loader.vertex_buffer(),
                    self.mesh_loader.normal_buffer(),
                    self.mesh_loader.uv_buffer(),
                );

                let shadow_cube = self.render_shadow_map.srv().gpu_descriptor(0);
                self.pixel_hlsl.set_roots(
                    &self.pixel_hlsl_bind,
                    &mut list,
                    self.mesh_instance_loader.instance_data_buffer(),
                    self.mesh_instance_loader.texture_metadata_buffer(),
                    self.texture_loader.textures_table(),
                    shadow_cube,
                );

                list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
                list.ia_set_index_buffer(self.mesh_loader.index_buffer());

                for model_handle in 0..entity_model.models.len() {
                    let entity_handle = entity_model.get_entity(model_handle);
                    let model = entity_model.get_model(model_handle);

                    /*
                    let texture_metadata = shaderbindings::STextureMetadata::new_from_model(&model);

                    let texture_gpu_descriptor = model.diffuse_texture.map(|handle| {
                        self.texture_loader.texture_gpu_descriptor(handle).unwrap()
                    });

                    let (verts_vbv, normals_vbv) = match entity_animation.get_skinning_for_entity(entity_handle) {
                        Some(skinning) => (&skinning.skinned_verts_vbv, &skinning.skinned_normals_vbv),
                        None => (self.mesh_loader.local_verts_vbv(model.mesh), self.mesh_loader.local_normals_vbv(model.mesh)),
                    };

                    self.vertex_hlsl.set_graphics_roots(
                        &self.vertex_hlsl_bind,
                        &mut list,
                        &shaderbindings::SModelViewProjection::new(&context.view_projection_matrix, &entities.get_entity_location(entity_handle)),
                    );
                    self.vertex_hlsl.set_vertex_buffers(
                        &mut list,
                        verts_vbv,
                        normals_vbv,
                        self.mesh_loader.uvs_vbv(model.mesh),
                    );
                    self.pixel_hlsl.set_graphics_roots(
                        &self.pixel_hlsl_bind,
                        &mut list,
                        texture_metadata,
                        texture_gpu_descriptor,
                        self.render_shadow_map.srv().gpu_descriptor(0),
                    );
                    */

                    self.mesh_loader.draw_one(model.mesh, &mut list, model.instance_location)?;
                }
            }

            // -- execute on the queue
            self.direct_command_pool.execute_and_free_list(&mut handle)?;

            Ok(())
        }
    }

    pub fn present(&mut self, window: &mut n12::SD3D12Window) -> Result<(), &'static str> {
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        let backbuffer = window.currentbackbuffer();

        // -- transition to present
        list.transition_resource(
            backbuffer,
            t12::EResourceStates::RenderTarget,
            t12::EResourceStates::Present,
        )?;

        drop(list);
        self.direct_command_pool.execute_and_free_list(&mut handle)?;

        self.frame_fence_values[window.currentbackbufferindex()] =
            self.direct_command_queue.borrow_mut().signal_internal_fence()?;

        // -- present the swap chain and switch to next buffer in swap chain
        window.present()?;

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), &'static str> {
        self.direct_command_queue.borrow_mut().flush_blocking()
    }
}

impl Drop for SRender {
    fn drop(&mut self) {
        self.flush().unwrap();

        self.shutdown();
    }
}

pub fn cast_ray_against_entity_model(data_bucket: &SDataBucket, ray: &SRay, entity: SEntityHandle) -> Option<f32> {
    let mut result = None;

    data_bucket.get::<SEntityBucket>()
        .and::<SRender>()
        .and::<entity_model::SBucket>()
        .with_ccc(|entities, render, em| {
            let entity_to_world = entities.get_entity_location(entity);
            let model_handle = em.handle_for_entity(entity).unwrap();
            result = render.ray_intersects(em.get_model(model_handle), &ray.origin, &ray.dir, &entity_to_world);
        });

    result
}

pub fn update_render_frame(game_context: &SGameContext, frame_context: &SFrameContext) {
    game_context.data_bucket.get::<SRender>()
        .and::<SEntityBucket>()
        .and::<entity_animation::SBucket>()
        .and::<entity_model::SBucket>()
        .and::<camera::SDebugFPCamera>()
        .with_mmmcc(|render, entities, entity_animation, entity_model, camera| {
            let view_matrix = camera.world_to_view_matrix();

            let render_result = render.render_frame(&game_context.window, &view_matrix, entities, entity_animation, entity_model, frame_context.imgui_draw_data);
            match render_result {
                Ok(_) => {},
                Err(e) => {
                    println!("ERROR: render failed with error '{}'", e);
                    panic!();
                },
            }
        });
}
