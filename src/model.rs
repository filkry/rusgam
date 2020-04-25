use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use glm::{Vec3, Vec2, Mat4};
use arrayvec::{ArrayString};

use t12;
use n12;
use allocate::{SMemVec, SYSTEM_ALLOCATOR};
use collections::{SStoragePool, SPoolHandle};
use safewindows;
use rustywindows;
use utils;
use utils::{STransform};

#[allow(dead_code)]
struct SVertexPosColourUV {
    position: Vec3,
    colour: Vec3,
    normal: Vec3,
    uv: Vec2,
}

// -- must match SVertexPosColorUV vertex.hlsl
pub fn model_per_vertex_input_layout_desc() -> t12::SInputLayoutDesc {
    let input_element_desc = [
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
            "NORMAL",
            0,
            t12::EDXGIFormat::R32G32B32Float,
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
    ];

    t12::SInputLayoutDesc::create(&input_element_desc)
}

#[allow(dead_code)]
pub struct SMesh<'a> {
    uid: u64,

    per_vertex_data: SMemVec<'a, SVertexPosColourUV>,
    pub(super) triangle_indices: SMemVec<'a, u16>,

    pub(super) vertex_buffer_resource: n12::SResource,
    pub(super) vertex_buffer_view: t12::SVertexBufferView,
    pub(super) index_buffer_resource: n12::SResource,
    pub(super) index_buffer_view: t12::SIndexBufferView,
}

pub struct STexture<'a> {
    uid: u64,

    #[allow(dead_code)] // maybe unnecessary?
    pub(super) srv_heap: &'a n12::descriptorallocator::SDescriptorAllocator,
    pub(super) _diffuse_texture_resource: Option<n12::SResource>,
    pub(super) diffuse_texture_srv: Option<n12::descriptorallocator::SDescriptorAllocatorAllocation<'a>>,
}

pub struct SMeshLoader<'a> {
    device: &'a n12::SDevice,
    copy_command_list_pool: n12::SCommandListPool<'a>,

    mesh_pool: SStoragePool<SMesh<'a>>,
}

pub struct STextureLoader<'a> {
    device: &'a n12::SDevice,
    copy_command_list_pool: n12::SCommandListPool<'a>,
    direct_command_list_pool: n12::SCommandListPool<'a>,
    srv_heap: &'a n12::descriptorallocator::SDescriptorAllocator,

    texture_pool: SStoragePool<STexture<'a>>,
}

pub struct SModel {
    pub mesh: SPoolHandle,

    // -- material info
    diffuse_colour: Vec3,
    diffuse_texture: Option<SPoolHandle>,
    diffuse_weight: f32,
}

// -- used to fill out shader metadata, must match STextureMetadata in pixel.hlsl
#[repr(C)]
pub struct STextureMetadata {
    diffuse_colour: Vec3,
    has_diffuse_texture: f32,
    diffuse_weight: f32,
}

impl<'a> SMeshLoader<'a> {
    pub fn new(
        device: &'a n12::SDevice,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: &'a RefCell<n12::SCommandQueue>,
        pool_id: u64,
        max_mesh_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device,
            copy_command_list_pool: n12::SCommandListPool::create(&device, &copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            mesh_pool: SStoragePool::create(pool_id, max_mesh_count),
        })
    }

    pub fn get_or_create_mesh(&mut self, asset_name: &'static str, tobj_mesh: &tobj::Mesh) -> Result<SPoolHandle, &'static str> {
        let uid = {
            let mut s = DefaultHasher::new();
            asset_name.hash(&mut s);
            s.finish()
        };

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.mesh_pool.used() {
            if let Some(mesh) = &self.mesh_pool.get_by_index(i as u16).unwrap() {
                if mesh.uid == uid {
                    return Ok(self.mesh_pool.handle_for_index(i as u16));
                }
            }
        }

        let mut vert_vec = SMemVec::<SVertexPosColourUV>::new(&SYSTEM_ALLOCATOR, tobj_mesh.positions.len(), 0).unwrap();
        let mut index_vec = SMemVec::<u16>::new(&SYSTEM_ALLOCATOR, tobj_mesh.indices.len(), 0).unwrap();

        assert!(tobj_mesh.positions.len() % 3 == 0);
        assert!(tobj_mesh.texcoords.len() / 2 == tobj_mesh.positions.len() / 3);
        assert!(tobj_mesh.normals.len() == tobj_mesh.positions.len());

        for vidx in 0..tobj_mesh.positions.len() / 3 {
            vert_vec.push(SVertexPosColourUV {
                position: Vec3::new(
                    tobj_mesh.positions[vidx * 3],
                    tobj_mesh.positions[vidx * 3 + 1],
                    tobj_mesh.positions[vidx * 3 + 2],
                ),
                uv: Vec2::new(
                    tobj_mesh.texcoords[vidx * 2],
                    tobj_mesh.texcoords[vidx * 2 + 1],
                ),
                normal: Vec3::new(
                    tobj_mesh.normals[vidx * 3],
                    tobj_mesh.normals[vidx * 3 + 1],
                    tobj_mesh.normals[vidx * 3 + 2],
                ),
                colour: Vec3::new(1.0, 1.0, 1.0),
            });
        }

        for idx in &tobj_mesh.indices {
            index_vec.push(*idx as u16);
        }

        // -- generate vertex/index resources and views
        let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
            let handle = self.copy_command_list_pool.alloc_list()?;
            let copycommandlist = self.copy_command_list_pool.get_list(handle)?;

            let mut vertbufferresource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(self.device, vert_vec.as_slice(), vertbufferflags)?
            };
            let vertexbufferview = vertbufferresource
                .destinationresource
                .create_vertex_buffer_view()?;

            let mut indexbufferresource = {
                let indexbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(self.device, index_vec.as_slice(), indexbufferflags)?
            };
            let indexbufferview = indexbufferresource
                .destinationresource
                .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

            let fenceval = self.copy_command_list_pool.execute_and_free_list(handle)?;
            self.copy_command_list_pool.wait_for_internal_fence_value(fenceval);
            self.copy_command_list_pool.free_allocators();
            assert_eq!(self.copy_command_list_pool.num_free_allocators(), 2);

            // -- debug
            unsafe {
                vertbufferresource.destinationresource.set_debug_name("vert dest");
                vertbufferresource.intermediateresource.set_debug_name("vert inter");
                indexbufferresource.destinationresource.set_debug_name("index dest");
                indexbufferresource.intermediateresource.set_debug_name("index inter");
            }

            (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
        };

        let mesh = SMesh{
            uid: uid,
            per_vertex_data: vert_vec,
            triangle_indices: index_vec,

            vertex_buffer_resource: vertbufferresource.destinationresource,
            vertex_buffer_view: vertexbufferview,
            index_buffer_resource: indexbufferresource.destinationresource,
            index_buffer_view: indexbufferview,
        };

        return self.mesh_pool.insert_val(mesh)
    }

    pub fn ray_intersects(
        &self,
        model: &SModel,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {

        let mesh = self.mesh_pool.get(model.mesh).unwrap();

        break_assert!(mesh.triangle_indices.len() % 3 == 0);
        let num_tris = mesh.triangle_indices.len() / 3;

        let mut min_t = None;

        for ti in 0..num_tris {
            let ti_vi_0 = mesh.triangle_indices[ti * 3 + 0];
            let ti_vi_1 = mesh.triangle_indices[ti * 3 + 1];
            let ti_vi_2 = mesh.triangle_indices[ti * 3 + 2];

            let v0_pos = &mesh.per_vertex_data[ti_vi_0 as usize].position;
            let v1_pos = &mesh.per_vertex_data[ti_vi_1 as usize].position;
            let v2_pos = &mesh.per_vertex_data[ti_vi_2 as usize].position;

            let v0_ray_space_pos = model_to_ray_space.mul_point(&v0_pos);
            let v1_ray_space_pos = model_to_ray_space.mul_point(&v1_pos);
            let v2_ray_space_pos = model_to_ray_space.mul_point(&v2_pos);

            if let Some(t) = utils::ray_intersects_triangle(
                &ray_origin,
                &ray_dir,
                &v0_ray_space_pos.xyz(),
                &v1_ray_space_pos.xyz(),
                &v2_ray_space_pos.xyz()) {

                if let Some(cur_min_t) = min_t {
                    if t < cur_min_t {
                        min_t = Some(t);
                    }
                }
                else {
                    min_t = Some(t);
                }
            }
        }

        return min_t;
    }

    pub fn render(
        &self,
        mesh_handle: SPoolHandle,
        cl: &mut n12::SCommandList,
        view_projection: &glm::Mat4,
        model_xform: &STransform,
    ) -> Result<(), &'static str> {
        let mesh = self.mesh_pool.get(mesh_handle)?;

        // -- assuming the same pipline state, root signature, viewport, scissor rect,
        // -- render target, for every model for now. These are set
        // -- outside of here

        // -- setup input assembler
        cl.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
        cl.ia_set_vertex_buffers(0, &[&mesh.vertex_buffer_view]);
        cl.ia_set_index_buffer(&mesh.index_buffer_view);

        #[allow(dead_code)]
        struct SModelViewProjection {
            model: Mat4,
            view_projection: Mat4,
            mvp: Mat4,
        }

        let model_matrix = model_xform.as_mat4();

        let mvp_matrix = view_projection * model_matrix;
        let mvp = SModelViewProjection{
            model: model_matrix.clone(),
            view_projection: view_projection.clone(),
            mvp: mvp_matrix,
        };

        cl.set_graphics_root_32_bit_constants(0, &mvp, 0);

        // -- draw
        cl.draw_indexed_instanced(mesh.triangle_indices.len() as u32, 1, 0, 0, 0);

        Ok(())
    }
}

impl<'a> STextureLoader<'a> {
    pub fn new(
        device: &'a n12::SDevice,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: &'a RefCell<n12::SCommandQueue>,
        direct_command_queue: &'a RefCell<n12::SCommandQueue>,
        srv_heap: &'a n12::SDescriptorAllocator,
        pool_id: u64,
        max_texture_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device,
            copy_command_list_pool: n12::SCommandListPool::create(&device, &copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            direct_command_list_pool: n12::SCommandListPool::create(&device, &direct_command_queue, &winapi.rawwinapi(), 1, 10)?,
            srv_heap,

            texture_pool: SStoragePool::create(pool_id, max_texture_count),
        })
    }

    pub fn get_or_create_texture(&mut self, texture_name: &String) -> Result<SPoolHandle, &'static str> {

        let uid = {
            let mut s = DefaultHasher::new();
            texture_name.hash(&mut s);
            s.finish()
        };

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.texture_pool.used() {
            if let Some(texture) = &self.texture_pool.get_by_index(i as u16)? {
                if texture.uid == uid {
                    return Ok(self.texture_pool.handle_for_index(i as u16));
                }
            }
        }

        let texture_resource = {
            let handle = self.copy_command_list_pool.alloc_list()?;
            let copycommandlist = self.copy_command_list_pool.get_list(handle)?;

            let mut texture_asset = ArrayString::<[_; 128]>::new();
            texture_asset.push_str("assets/");
            texture_asset.push_str(texture_name);
            let (mut _intermediate_resource, mut resource) = n12::load_texture(self.device, copycommandlist, texture_asset.as_str());

            let fenceval = self.copy_command_list_pool.execute_and_free_list(handle)?;
            self.copy_command_list_pool.wait_for_internal_fence_value(fenceval);
            self.copy_command_list_pool.free_allocators();
            assert_eq!(self.copy_command_list_pool.num_free_allocators(), 2);

            unsafe {
                _intermediate_resource.set_debug_name("text inter");
                resource.set_debug_name("text dest");
            }

            Some(resource)
        };

        // -- transition texture to PixelShaderResource
        {
            let handle = self.direct_command_list_pool.alloc_list()?;
            let list = self.direct_command_list_pool.get_list(handle)?;

            list.transition_resource(
                &texture_resource.as_ref().unwrap(),
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::PixelShaderResource,
            )
            .unwrap();

            let fenceval = self.direct_command_list_pool.execute_and_free_list(handle)?;
            self.direct_command_list_pool.wait_for_internal_fence_value(fenceval);
        }

        // -- get texture SRV
        let texture_srv = {
            let srv_desc = t12::SShaderResourceViewDesc {
                format: t12::EDXGIFormat::R8G8B8A8UNorm,
                view: t12::ESRV::Texture2D {
                    data: t12::STex2DSRV {
                        mip_levels: 1,
                        ..Default::default()
                    },
                },
            };

            let descriptors = self.srv_heap.alloc(1)?;
            self.device.create_shader_resource_view(
                texture_resource.as_ref().unwrap(),
                &srv_desc,
                descriptors.cpu_descriptor(0),
            )?;

            Some(descriptors)
        };

        let texture = STexture{
            uid: uid,
            srv_heap: self.srv_heap,
            _diffuse_texture_resource: texture_resource,
            diffuse_texture_srv: texture_srv,
        };

        self.texture_pool.insert_val(texture)
    }

    pub fn texture_gpu_descriptor(&self, texture: SPoolHandle) -> Result<t12::SGPUDescriptorHandle, &'static str> {
        let texture = self.texture_pool.get(texture)?;
        if let Some(srv) = &texture.diffuse_texture_srv {
            return Ok(srv.gpu_descriptor(0))
        }

        return Err("Tried to get descriptor for invalid SRV.")
    }
}

impl SModel {

    pub fn new_from_obj(
        obj_file: &'static str,
        mesh_loader: &mut SMeshLoader,
        texture_loader: &mut STextureLoader,
        diffuse_weight: f32,
    ) -> Result<Self, &'static str> {

        let (models, materials) = tobj::load_obj(&std::path::Path::new(obj_file)).unwrap();
        assert_eq!(models.len(), 1);

        let mesh = mesh_loader.get_or_create_mesh(obj_file, &models[0].mesh);
        let mut diffuse_colour = Vec3::new(0.0, 0.0, 0.0);
        let mut diffuse_texture : Option<SPoolHandle> = None;

        if materials.len() > 0 {
            assert_eq!(materials.len(), 1);

            diffuse_colour[0] = materials[0].diffuse[0];
            diffuse_colour[1] = materials[0].diffuse[1];
            diffuse_colour[2] = materials[0].diffuse[2];

            if materials[0].diffuse_texture.len() > 0 {
                diffuse_texture = Some(texture_loader.get_or_create_texture(&materials[0].diffuse_texture)?)
            }
        }

        Ok(Self {
            mesh: mesh?,

            // -- material info
            diffuse_colour,
            diffuse_texture,
            diffuse_weight,
        })
    }

    pub fn set_texture_root_parameters(
        &self,
        texture_loader: &STextureLoader,
        cl: &mut n12::SCommandList,
        metadata_constant_root_parameter: u32,
        texture_descriptor_table_root_parameter: usize,
    ) {
        let mut texture_metadata = STextureMetadata{
            diffuse_colour: self.diffuse_colour,
            has_diffuse_texture: 0.0,
            diffuse_weight: self.diffuse_weight,
        };

        if let Some(texture) = self.diffuse_texture {
            texture_metadata.has_diffuse_texture = 1.0;
            cl.set_graphics_root_descriptor_table(
                texture_descriptor_table_root_parameter,
                &texture_loader.texture_gpu_descriptor(texture).unwrap(),
            );
        }

        cl.set_graphics_root_32_bit_constants(metadata_constant_root_parameter, &texture_metadata, 0);
    }

}