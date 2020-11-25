// -- std includes
use std::cell::RefCell;
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
use databucket::{SDataBucket};
use entity::{SEntityBucket, SEntityHandle};
use entity_model;
use model::{SModel, SModelSkinning, SMeshLoader, STextureLoader};
use safewindows;
use rustywindows;
use utils;
use utils::{STransform, SRay};

mod compute_skinning_pipeline;
mod shadowmapping;
mod render_imgui;
pub mod temp;
pub mod shaderbindings;

use self::render_imgui::{SRenderImgui};
use self::temp::{SRenderTemp};

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

pub struct SRender<'a> {
    factory: n12::SFactory,
    _adapter: n12::SAdapter, // -- maybe don't need to keep
    device: Rc<n12::SDevice>,

    direct_command_pool: n12::SCommandListPool,
    copy_command_pool: n12::SCommandListPool,
    _copy_command_queue: Rc<RefCell<n12::SCommandQueue>>,
    direct_command_queue: Rc<RefCell<n12::SCommandQueue>>,

    dsv_heap: Rc<n12::SDescriptorAllocator>,
    cbv_srv_uav_heap: Rc<n12::SDescriptorAllocator>,

    mesh_loader: SMeshLoader<'a>,
    texture_loader: STextureLoader,

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
    render_imgui: SRenderImgui<'a>,
    render_temp: SRenderTemp<'a>,
}

pub fn compile_shaders_if_changed() {
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

                let mut command = std::process::Command::new("externals/dxc_2020_10-22/bin/x64/dxc.exe");

                command.arg("-E").arg("main") // -- entry point
                       .arg("-T").arg(type_) // -- type
                       .arg(shader_src_path)
                       .arg("-Od") // -- optimization level debug
                       .arg("-Zi") // -- enable debug information
                       .arg("-WX") // -- treat warnings as errors
                       .arg("-Qembed_debug") // -- embed PDB in shader container (must be used with Zi)
                       .arg("-Fo").arg(built_shader_path); // -- output object file

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
        let mut mesh_loader = SMeshLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), Rc::downgrade(&cbv_srv_uav_heap), 23948934, 1024)?;
        let mut texture_loader = STextureLoader::new(Rc::downgrade(&device), &winapi, Rc::downgrade(&copy_command_queue), Rc::downgrade(&direct_command_queue), Rc::downgrade(&cbv_srv_uav_heap), 9323, 1024)?;

        // -- load shaders
        let vertex_hlsl = shaderbindings::SVertexHLSL::new()?;
        let pixel_hlsl = shaderbindings::SPixelHLSL::new()?;

        // -- root signature stuff
        let mut input_layout_desc = shaderbindings::SVertexHLSL::input_layout_desc();

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
        let pipeline_state_stream = SPipelineStateStream {
            root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
            input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
            primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
                t12::EPrimitiveTopologyType::Triangle,
            ),
            vertex_shader: n12::SPipelineStateStreamVertexShader::create(vertex_hlsl.bytecode()),
            pixel_shader: n12::SPipelineStateStreamPixelShader::create(pixel_hlsl.bytecode()),
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

    pub fn temp(&mut self) -> &mut SRenderTemp<'a> {
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

    pub fn new_model_from_obj(&mut self, obj_file_path: &'static str, diffuse_weight: f32, is_lit: bool) -> Result<SModel, &'static str> {
        SModel::new_from_obj(obj_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight, is_lit)
    }

    pub fn new_model_from_gltf(&mut self, gltf_file_path: &'static str, diffuse_weight: f32, is_lit: bool) -> Result<SModel, &'static str> {
        SModel::new_from_gltf(gltf_file_path, &mut self.mesh_loader, &mut self.texture_loader, diffuse_weight, is_lit)
    }

    pub fn mesh_loader(&self) -> &SMeshLoader {
        &self.mesh_loader
    }

    pub fn bind_model_skinning(&mut self, model: &SModel) -> Result<SModelSkinning, &'static str> {
        self.mesh_loader.bind_skinning(model.mesh)
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
        window: &mut n12::SD3D12Window,
        view_matrix: &Mat4,
        entities: &mut SEntityBucket,
        entity_model: &entity_model::SBucket,
        world_models: &[SModel],
        world_model_xforms: &[STransform],
        imgui_draw_data: Option<&imgui::DrawData>,
    ) -> Result<(), &'static str> {
        let back_buffer_idx = window.currentbackbufferindex();

        // -- wait for buffer to be available
        self.direct_command_queue.borrow()
            .wait_for_internal_fence_value(self.frame_fence_values[back_buffer_idx]);

        // -- clear RTV and depth buffer, transition
        {
            let backbuffer = window.currentbackbuffer();

            let mut handle = self.direct_command_pool.alloc_list()?;
            let mut list = self.direct_command_pool.get_list(&handle)?;

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
            list.clear_depth_stencil_view(self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;

            drop(list);
            self.direct_command_pool.execute_and_free_list(&mut handle)?;
        }

        // -- kick off imgui copies
        if let Some(idd) = imgui_draw_data {
            self.setup_imgui_draw_data_resources(window, idd)?;
        }

        // -- update skinned buffers
        self.compute_skinning(entities, entity_model)?;

        // -- $$$FRK(TODO): should initialize the shadow map depth buffer to empty, so we still get light if we don't render maps
        self.render_shadow_maps(world_models, world_model_xforms)?;
        self.render_world(window, view_matrix, entities, entity_model)?;
        self.render_temp_in_world(window, view_matrix)?;

        // -- clear depth buffer again
        {
            let mut handle = self.direct_command_pool.alloc_list()?;
            let mut list = self.direct_command_pool.get_list(&handle)?;
            list.clear_depth_stencil_view(self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0), 1.0)?;
            drop(list);
            self.direct_command_pool.execute_and_free_list(&mut handle)?;
        }

        self.render_temp_over_world(window, view_matrix)?;
        if let Some(idd) = imgui_draw_data {
            self.render_imgui(window, idd)?;
        }
        self.present(window)?;

        self.render_temp.clear_tables_without_tokens();

        Ok(())
    }

    fn compute_skinning(&mut self, entities: &mut SEntityBucket, entity_model: &entity_model::SBucket) -> Result<(), &'static str> {
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        self.compute_skinning_pipeline.compute(list.deref_mut(), &self.mesh_loader, entities, entity_model);

        drop(list);

        let fence_val = self.direct_command_pool.execute_and_free_list(&mut handle)?;
        self.direct_command_pool.wait_for_internal_fence_value(fence_val); // $$$FRK(TODO) reconsider this fence

        Ok(())
    }

    pub fn render_shadow_maps(
        &mut self,
        world_models: &[SModel],
        world_model_xforms: &[STransform],
    ) -> Result<(), &'static str> {
        let mut handle = self.direct_command_pool.alloc_list()?;
        let mut list = self.direct_command_pool.get_list(&handle)?;

        self.render_shadow_map.render(
            &self.mesh_loader,
            &Vec3::new(5.0, 5.0, 5.0),
            list.deref_mut(),
            world_models,
            world_model_xforms,
        )?;

        drop(list);

        let fence_val = self.direct_command_pool.execute_and_free_list(&mut handle)?;
        self.direct_command_pool.wait_for_internal_fence_value(fence_val); // $$$FRK(TODO) reconsider this fence

        Ok(())
    }

    pub fn render_world(&mut self, window: &mut n12::SD3D12Window, view_matrix: &Mat4, entities: &SEntityBucket, entity_model: &entity_model::SBucket) -> Result<(), &'static str> {
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

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let mut handle = self.direct_command_pool.alloc_list()?;

            {
                let mut list = self.direct_command_pool.get_list(&handle)?;

                let render_target_view = window.currentrendertargetdescriptor()?;
                let depth_texture_view = self.depth_texture_view.as_ref().expect("no depth texture").cpu_descriptor(0);

                // -- set up pipeline
                list.set_pipeline_state(&self.pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&self.root_signature.raw());

                // -- setup rasterizer state
                list.rs_set_viewports(&[&viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&self.scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

                self.cbv_srv_uav_heap.with_raw_heap(|rh| {
                    list.set_descriptor_heaps(&[rh]);
                });

                let view_perspective = perspective_matrix * view_matrix;
                assert!(false, "Should iterate over models");
                for ei in 0..entities.entities().max() {
                    let entity_handle_res = entities.entities().handle_for_index(ei);
                    if entity_handle_res.is_err() {
                        continue;
                    }
                    let entity_handle = entity_handle_res.expect("checked above");

                    let model_handle = entity_model.handle_for_entity(entity_handle).unwrap();
                    let model = entity_model.get_model(model_handle);

                    let texture_metadata = shaderbindings::STextureMetadata::new_from_model(&model);

                    let texture_gpu_descriptor = model.diffuse_texture.map(|handle| {
                        self.texture_loader.texture_gpu_descriptor(handle).unwrap()
                    });

                    let (verts_vbv, normals_vbv) = match entities.get_model_skinning(entity_handle) {
                        Some(skinning) => (&skinning.skinned_verts_vbv, &skinning.skinned_normals_vbv),
                        None => (self.mesh_loader.local_verts_vbv(model.mesh), self.mesh_loader.local_normals_vbv(model.mesh)),
                    };

                    self.vertex_hlsl.set_graphics_roots(
                        &self.vertex_hlsl_bind,
                        &mut list,
                        &shaderbindings::SModelViewProjection::new(&view_perspective, &entities.get_entity_location(entity_handle)),
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

                    self.mesh_loader.set_index_buffer_and_draw(model.mesh, &mut list)?;
                }
            }

            // -- execute on the queue
            assert_eq!(window.currentbackbufferindex(), backbufferidx);
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

impl<'a> Drop for SRender<'a> {
    fn drop(&mut self) {
        self.flush().unwrap();

        self.shutdown();
    }
}

pub fn cast_ray_against_entity_model(data_bucket: &SDataBucket, ray: &SRay, entity: SEntityHandle) -> Option<f32> {
    let mut result = None;

    data_bucket.get::<SEntityBucket>().unwrap()
        .and::<SRender>(data_bucket).unwrap()
        .and::<entity_model::SBucket>(data_bucket).unwrap()
        .with_ccc(|entities: &SEntityBucket, render: &SRender, em: &entity_model::SBucket| {
            let entity_to_world = entities.get_entity_location(entity);
            let model_handle = em.handle_for_entity(entity).unwrap();
            result = render.ray_intersects(em.get_model(model_handle), &ray.origin, &ray.dir, &entity_to_world);
        });

    result
}


