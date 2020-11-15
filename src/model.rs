use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Weak;

use glm::{Vec4, Vec3, Vec2, Mat4};
use arrayvec::{ArrayString};
use gltf;

use t12;
use n12;
use n12::descriptorallocator::{descriptor_alloc};
use allocate::{SMemVec, SYSTEM_ALLOCATOR};
use collections;
use collections::{SStoragePool};
use safewindows;
use render::shaderbindings;
use rustywindows;
use utils;
use utils::{STransform};

struct SMeshSkinning<'a> {
    vertex_skin_data: SMemVec<'a, shaderbindings::SVertexSkinningData>,
    vertex_skinning_buffer_resource: n12::SResource,
    vertex_skinning_buffer_view: t12::SVertexBufferView,

    model_to_joint_xforms: SMemVec<'a, Mat4>,
    model_to_joint_xforms_resource: n12::SResource,
    model_to_joint_xforms_view: n12::SDescriptorAllocatorAllocation,
}

#[allow(dead_code)]
pub struct SMesh<'a> {
    uid: u64,

    per_vertex_data: SMemVec<'a, shaderbindings::SBaseVertexData>,
    local_aabb: utils::SAABB,
    pub(super) triangle_indices: SMemVec<'a, u16>,

    pub(super) vertex_buffer_resource: n12::SResource,
    pub(super) vertex_buffer_view: t12::SVertexBufferView,
    pub(super) index_buffer_resource: n12::SResource,
    pub(super) index_buffer_view: t12::SIndexBufferView,

    //skinning: Option<SMeshSkinning<'a>>,
}

pub struct STexture {
    uid: Option<u64>, // if the texture is unique, it will have no ID

    #[allow(dead_code)] // maybe unnecessary?
    //pub(super) srv_heap: &'a n12::descriptorallocator::SDescriptorAllocator,
    pub(super) _diffuse_texture_resource: Option<n12::SResource>,
    pub(super) diffuse_texture_srv: Option<n12::descriptorallocator::SDescriptorAllocatorAllocation>,
}

pub struct SMeshLoader<'a> {
    device: Weak<n12::SDevice>,
    copy_command_list_pool: n12::SCommandListPool,
    direct_command_list_pool: n12::SCommandListPool,

    mesh_pool: SStoragePool<SMesh<'a>, u16, u16>,
}
pub type SMeshHandle = collections::SPoolHandle<u16, u16>;

pub struct STextureLoader {
    device: Weak<n12::SDevice>,
    copy_command_list_pool: n12::SCommandListPool,
    direct_command_list_pool: n12::SCommandListPool,
    srv_heap: Weak<n12::descriptorallocator::SDescriptorAllocator>,

    texture_pool: SStoragePool<STexture, u16, u16>,
}
pub type STextureHandle = collections::SPoolHandle<u16, u16>;

#[derive(Clone, Copy)]
pub struct SModel {
    pub mesh: SMeshHandle,

    pub pickable: bool,

    // -- material info
    pub diffuse_colour: Vec4,
    pub diffuse_texture: Option<STextureHandle>,
    pub diffuse_weight: f32,
    pub is_lit: bool,
}

impl<'a> SMeshLoader<'a> {
    pub fn new(
        device: Weak<n12::SDevice>,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        direct_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        pool_id: u64,
        max_mesh_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device: device.clone(),
            copy_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("bad device").deref(), copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            direct_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("bad device").deref(), direct_command_queue, &winapi.rawwinapi(), 1, 2)?,
            mesh_pool: SStoragePool::create(pool_id, max_mesh_count),
        })
    }

    pub fn get_or_create_mesh_gltf(&mut self, asset_file_path: &'static str, gltf_data: &gltf::Gltf) -> Result<SMeshHandle, &'static str> {
        let uid = {
            let mut s = DefaultHasher::new();
            asset_file_path.hash(&mut s);
            s.finish()
        };

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.mesh_pool.used() {
            if let Some(mesh) = &self.mesh_pool.get_by_index(i as u16).unwrap() {
                if mesh.uid == uid {
                    return Ok(self.mesh_pool.handle_for_index(i as u16)?);
                }
            }
        }

        assert!(gltf_data.buffers().len() == 1, "can't handle multi-buffer gltf currently");
        let buffer = gltf_data.buffers().nth(0).unwrap();
        let buffer_bytes : Vec<u8> = {
            if let gltf::buffer::Source::Uri(binname) = buffer.source() {
                let path = std::path::Path::new("./assets/");
                let binname = std::path::Path::new(binname);
                let fullpath = path.join(binname);
                println!("Reading GLTF from path: {:?}", fullpath);
                std::fs::read(fullpath).unwrap()
            }
            else {
                panic!("Expected external buffer!");
            }
        };

        assert!(gltf_data.meshes().len() == 1, "Can't handle multi-mesh model currently");
        let mesh = gltf_data.meshes().nth(0).unwrap();

        assert!(mesh.primitives().len() == 1, "can't handle multi-primitive mesh currently");
        let primitive = mesh.primitives().nth(0).unwrap();

        fn primitive_semantic_slice<'a, T>(
            primitive: &gltf::Primitive,
            semantic: &gltf::mesh::Semantic,
            expected_datatype: gltf::accessor::DataType,
            expected_dimensions: gltf::accessor::Dimensions,
            bytes: &'a Vec<u8>,
        ) -> &'a [T] {
            let accessor = primitive.get(semantic).unwrap();
            assert!(accessor.data_type() == expected_datatype);
            assert!(accessor.dimensions() == expected_dimensions);

            let size = accessor.size();
            assert!(size == std::mem::size_of::<T>());
            let count = accessor.count();

            let view = accessor.view().unwrap();
            assert!(view.stride().is_none());

            let slice_bytes = &bytes[view.offset()..(view.offset() + size * count)];
            let (_a, result, _b) = unsafe { slice_bytes.align_to::<T>() };
            assert!(_a.len () == 0 && _b.len() == 0);

            result
        }

        let positions : &[Vec3] = primitive_semantic_slice(
            &primitive,
            &gltf::mesh::Semantic::Positions,
            gltf::accessor::DataType::F32,
            gltf::accessor::Dimensions::Vec3,
            &buffer_bytes,
        );
        let normals : &[Vec3] = primitive_semantic_slice(
            &primitive,
            &gltf::mesh::Semantic::Normals,
            gltf::accessor::DataType::F32,
            gltf::accessor::Dimensions::Vec3,
            &buffer_bytes,
        );

        /*
        let positions = {
            let positions_accessor = primitive.get(&gltf::mesh::Semantic::Positions).unwrap();
            assert!(positions_accessor.data_type() == gltf::accessor::DataType::F32);
            assert!(positions_accessor.dimensions() == gltf::accessor::Dimensions::Vec4);

            let vert_size = positions_accessor.size();
            assert!(vert_size == std::mem::size_of::<Vec4>());
            let num_verts = positions_accessor.count();

            let positions_view = positions_accessor.view().unwrap();
            assert!(positions_view.stride().is_none());

            let positions_bytes = &buffer_bytes[positions_view.offset()..(positions_view.offset() + vert_size * num_verts)];
            let (_a, positions, _b) = unsafe { positions_bytes.align_to::<Vec4>() };
            assert!(_a.len () == 0 && _b.len() == 0);

            positions
        };
        */
        assert!(positions.len() == normals.len());

        let mut vert_vec = SMemVec::<shaderbindings::SBaseVertexData>::new(&SYSTEM_ALLOCATOR, positions.len(), 0).unwrap();
        for i in 0..positions.len() {
            vert_vec.push(shaderbindings::SBaseVertexData{
                position: Vec3::new(positions[i].x, positions[i].y, positions[i].z),
                normal: Vec3::new(normals[i].x, normals[i].y, normals[i].z),
                uv: Vec2::new(0.0, 0.0),
            });
        }

        /*
        let mut index_vec = SMemVec::<u16>::new(&SYSTEM_ALLOCATOR, tobj_mesh.indices.len(), 0).unwrap();

        assert!(tobj_mesh.positions.len() % 3 == 0);
        assert!(tobj_mesh.texcoords.len() / 2 == tobj_mesh.positions.len() / 3);
        assert!(tobj_mesh.normals.len() == tobj_mesh.positions.len());

        for vidx in 0..tobj_mesh.positions.len() / 3 {
            vert_vec.push(shaderbindings::SBaseVertexData {
                position: Vec3::new(
                    tobj_mesh.positions[vidx * 3],
                    tobj_mesh.positions[vidx * 3 + 1],
                    tobj_mesh.positions[vidx * 3 + 2],
                ),
                normal: Vec3::new(
                    tobj_mesh.normals[vidx * 3],
                    tobj_mesh.normals[vidx * 3 + 1],
                    tobj_mesh.normals[vidx * 3 + 2],
                ),
                uv: Vec2::new(
                    tobj_mesh.texcoords[vidx * 2],
                    tobj_mesh.texcoords[vidx * 2 + 1],
                ),
            });
        }

        for idx in &tobj_mesh.indices {
            index_vec.push(*idx as u16);
        }

        // -- generate vertex/index resources and views
        let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
            let mut handle = self.copy_command_list_pool.alloc_list()?;
            let mut copycommandlist = self.copy_command_list_pool.get_list(&handle)?;

            let mut vertbufferresource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(
                    self.device.upgrade().expect("device dropped").deref(),
                    vert_vec.as_slice(),
                    vertbufferflags
                )?
            };
            let vertexbufferview = vertbufferresource
                .destinationresource
                .create_vertex_buffer_view()?;

            let mut indexbufferresource = {
                let indexbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(
                    self.device.upgrade().expect("device dropped").deref(),
                    index_vec.as_slice(),
                    indexbufferflags
                )?
            };
            let indexbufferview = indexbufferresource
                .destinationresource
                .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

            drop(copycommandlist);

            let fence_val = self.copy_command_list_pool.execute_and_free_list(&mut handle)?;
            drop(handle);

            // -- $$$FRK(TODO): we have to wait here because we're going to drop the intermediate resource
            self.copy_command_list_pool.wait_for_internal_fence_value(fence_val);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_list_pool.gpu_wait(
                self.copy_command_list_pool.get_internal_fence(),
                fence_val,
            )?;

            // -- transition resources
            let mut handle  = self.direct_command_list_pool.alloc_list()?;
            let mut direct_command_list = self.direct_command_list_pool.get_list(&handle)?;

            direct_command_list.transition_resource(
                &vertbufferresource.destinationresource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::VertexAndConstantBuffer,
            )?;
            direct_command_list.transition_resource(
                &indexbufferresource.destinationresource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::IndexBuffer,
            )?;

            drop(direct_command_list);
            self.direct_command_list_pool.execute_and_free_list(&mut handle)?;

            // -- debug
            unsafe {
                vertbufferresource.destinationresource.set_debug_name("vert dest");
                vertbufferresource.intermediateresource.set_debug_name("vert inter");
                indexbufferresource.destinationresource.set_debug_name("index dest");
                indexbufferresource.intermediateresource.set_debug_name("index inter");
            }

            (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
        };

        let mut local_aabb = utils::SAABB::new(&vert_vec[0].position);
        for vi in 1..vert_vec.len() {
            local_aabb.expand(&vert_vec[vi].position);
        }
        //println!("Asset name: {}\nAABB: {:?}", asset_name, local_aabb);

        let mesh = SMesh{
            uid: uid,
            per_vertex_data: vert_vec,
            triangle_indices: index_vec,
            local_aabb: local_aabb,

            vertex_buffer_resource: vertbufferresource.destinationresource,
            vertex_buffer_view: vertexbufferview,
            index_buffer_resource: indexbufferresource.destinationresource,
            index_buffer_view: indexbufferview,
        };

        return self.mesh_pool.insert_val(mesh)
        */

        panic!("Not implemented.");
    }

    pub fn get_or_create_mesh_obj(&mut self, asset_name: &'static str, tobj_mesh: &tobj::Mesh) -> Result<SMeshHandle, &'static str> {
        let uid = {
            let mut s = DefaultHasher::new();
            asset_name.hash(&mut s);
            s.finish()
        };

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.mesh_pool.used() {
            if let Some(mesh) = &self.mesh_pool.get_by_index(i as u16).unwrap() {
                if mesh.uid == uid {
                    return Ok(self.mesh_pool.handle_for_index(i as u16)?);
                }
            }
        }

        let mut vert_vec = SMemVec::<shaderbindings::SBaseVertexData>::new(&SYSTEM_ALLOCATOR, tobj_mesh.positions.len(), 0).unwrap();
        let mut index_vec = SMemVec::<u16>::new(&SYSTEM_ALLOCATOR, tobj_mesh.indices.len(), 0).unwrap();

        assert!(tobj_mesh.positions.len() % 3 == 0);
        assert!(tobj_mesh.texcoords.len() / 2 == tobj_mesh.positions.len() / 3);
        assert!(tobj_mesh.normals.len() == tobj_mesh.positions.len());

        for vidx in 0..tobj_mesh.positions.len() / 3 {
            vert_vec.push(shaderbindings::SBaseVertexData {
                position: Vec3::new(
                    tobj_mesh.positions[vidx * 3],
                    tobj_mesh.positions[vidx * 3 + 1],
                    tobj_mesh.positions[vidx * 3 + 2],
                ),
                normal: Vec3::new(
                    tobj_mesh.normals[vidx * 3],
                    tobj_mesh.normals[vidx * 3 + 1],
                    tobj_mesh.normals[vidx * 3 + 2],
                ),
                uv: Vec2::new(
                    tobj_mesh.texcoords[vidx * 2],
                    tobj_mesh.texcoords[vidx * 2 + 1],
                ),
            });
        }

        for idx in &tobj_mesh.indices {
            index_vec.push(*idx as u16);
        }

        // -- generate vertex/index resources and views
        let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
            let mut handle = self.copy_command_list_pool.alloc_list()?;
            let mut copycommandlist = self.copy_command_list_pool.get_list(&handle)?;

            let mut vertbufferresource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(
                    self.device.upgrade().expect("device dropped").deref(),
                    vert_vec.as_slice(),
                    vertbufferflags
                )?
            };
            let vertexbufferview = vertbufferresource
                .destinationresource
                .create_vertex_buffer_view()?;

            let mut indexbufferresource = {
                let indexbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(
                    self.device.upgrade().expect("device dropped").deref(),
                    index_vec.as_slice(),
                    indexbufferflags
                )?
            };
            let indexbufferview = indexbufferresource
                .destinationresource
                .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

            drop(copycommandlist);

            let fence_val = self.copy_command_list_pool.execute_and_free_list(&mut handle)?;
            drop(handle);

            // -- $$$FRK(TODO): we have to wait here because we're going to drop the intermediate resource
            self.copy_command_list_pool.wait_for_internal_fence_value(fence_val);

            // -- have the direct queue wait on the copy upload to complete
            self.direct_command_list_pool.gpu_wait(
                self.copy_command_list_pool.get_internal_fence(),
                fence_val,
            )?;

            // -- transition resources
            let mut handle  = self.direct_command_list_pool.alloc_list()?;
            let mut direct_command_list = self.direct_command_list_pool.get_list(&handle)?;

            direct_command_list.transition_resource(
                &vertbufferresource.destinationresource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::VertexAndConstantBuffer,
            )?;
            direct_command_list.transition_resource(
                &indexbufferresource.destinationresource,
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::IndexBuffer,
            )?;

            drop(direct_command_list);
            self.direct_command_list_pool.execute_and_free_list(&mut handle)?;

            // -- debug
            unsafe {
                vertbufferresource.destinationresource.set_debug_name("vert dest");
                vertbufferresource.intermediateresource.set_debug_name("vert inter");
                indexbufferresource.destinationresource.set_debug_name("index dest");
                indexbufferresource.intermediateresource.set_debug_name("index inter");
            }

            (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
        };

        let mut local_aabb = utils::SAABB::new(&vert_vec[0].position);
        for vi in 1..vert_vec.len() {
            local_aabb.expand(&vert_vec[vi].position);
        }
        //println!("Asset name: {}\nAABB: {:?}", asset_name, local_aabb);

        let mesh = SMesh{
            uid: uid,
            per_vertex_data: vert_vec,
            triangle_indices: index_vec,
            local_aabb: local_aabb,

            vertex_buffer_resource: vertbufferresource.destinationresource,
            vertex_buffer_view: vertexbufferview,
            index_buffer_resource: indexbufferresource.destinationresource,
            index_buffer_view: indexbufferview,
        };

        return self.mesh_pool.insert_val(mesh)
    }

    pub fn get_mesh_local_aabb(&self, mesh: SMeshHandle) -> &utils::SAABB {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        &mesh.local_aabb
    }

    pub fn get_per_vertex_data(&self, mesh: SMeshHandle) -> &SMemVec<'a, shaderbindings::SBaseVertexData> {
        &self.mesh_pool.get(mesh).unwrap().per_vertex_data
    }

    #[allow(dead_code)]
    pub fn ray_intersects(
        &self,
        mesh: SMeshHandle,
        ray_origin: &Vec3,
        ray_dir: &Vec3,
        model_to_ray_space: &STransform,
    ) -> Option<f32> {
        let mesh = self.mesh_pool.get(mesh).unwrap();

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

    pub fn index_count(&self, mesh_handle: SMeshHandle) -> usize {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        mesh.triangle_indices.len()
    }

    pub fn vertex_buffer_view(&self, mesh_handle: SMeshHandle) -> &t12::SVertexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.vertex_buffer_view
    }

    pub fn index_buffer_view(&self, mesh_handle: SMeshHandle) -> &t12::SIndexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.index_buffer_view
    }

    pub fn bind_buffers_and_draw(
        &self,
        mesh_handle: SMeshHandle,
        cl: &mut n12::SCommandList,
    ) -> Result<(), &'static str> {
        let mesh = self.mesh_pool.get(mesh_handle)?;

        cl.ia_set_vertex_buffers(0, &[&mesh.vertex_buffer_view]);
        cl.ia_set_index_buffer(&mesh.index_buffer_view);
        cl.draw_indexed_instanced(mesh.triangle_indices.len() as u32, 1, 0, 0, 0);

        Ok(())
    }

    pub fn render(
        &self,
        mesh_handle: SMeshHandle,
        cl: &mut n12::SCommandList,
    ) -> Result<(), &'static str> {
        // -- assuming the same pipline state, root signature, viewport, scissor rect,
        // -- render target, for every model for now. These are set
        // -- outside of here

        // -- setup input assembler
        cl.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);

        // -- draw
        self.bind_buffers_and_draw(mesh_handle, cl)?;

        Ok(())
    }
}

impl STextureLoader {
    pub fn new(
        device: Weak<n12::SDevice>,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        direct_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        srv_heap: Weak<n12::SDescriptorAllocator>,
        pool_id: u64,
        max_texture_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device: device.clone(),
            copy_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("dropped device").deref(), copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            direct_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("dropped device").deref(), direct_command_queue, &winapi.rawwinapi(), 1, 10)?,
            srv_heap,

            texture_pool: SStoragePool::create(pool_id, max_texture_count),
        })
    }

    pub fn shutdown(&mut self) {
        self.texture_pool.clear();
    }

    pub fn create_texture_rgba32_from_resource(&mut self, uid: Option<u64>, texture_resource: Option<n12::SResource>) -> Result<STextureHandle, &'static str> {
        // -- transition texture to PixelShaderResource
        {
            let mut handle = self.direct_command_list_pool.alloc_list()?;
            let mut list = self.direct_command_list_pool.get_list(&handle)?;

            list.transition_resource(
                &texture_resource.as_ref().unwrap(),
                t12::EResourceStates::CopyDest,
                t12::EResourceStates::PixelShaderResource,
            )
            .unwrap();

            drop(list);

            let fenceval = self.direct_command_list_pool.execute_and_free_list(&mut handle)?;
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

            let descriptors = descriptor_alloc(&self.srv_heap.upgrade().expect("allocator dropped"), 1)?;
            self.device.upgrade().expect("device dropped").create_shader_resource_view(
                texture_resource.as_ref().unwrap(),
                &srv_desc,
                descriptors.cpu_descriptor(0),
            )?;

            Some(descriptors)
        };

        let texture = STexture{
            uid: uid,
            //srv_heap: self.srv_heap,
            _diffuse_texture_resource: texture_resource,
            diffuse_texture_srv: texture_srv,
        };

        self.texture_pool.insert_val(texture)

    }

    pub fn create_texture_rgba32_from_bytes(&mut self, width: u32, height: u32, data: &[u8]) -> Result<STextureHandle, &'static str> {
        let texture_resource = {
            let mut handle = self.copy_command_list_pool.alloc_list()?;
            let mut copycommandlist = self.copy_command_list_pool.get_list(&handle)?;

            let (mut _intermediate_resource, mut resource) = n12::load_texture_rgba32_from_bytes(
                self.device.upgrade().expect("dropped device").deref(),
                copycommandlist.deref_mut(),
                width,
                height,
                data,
            );

            drop(copycommandlist);

            let fenceval = self.copy_command_list_pool.execute_and_free_list(&mut handle)?;
            self.copy_command_list_pool.wait_for_internal_fence_value(fenceval);
            self.copy_command_list_pool.free_allocators();
            assert_eq!(self.copy_command_list_pool.num_free_allocators(), 2);

            unsafe {
                _intermediate_resource.set_debug_name("text inter");
                resource.set_debug_name("text dest");
            }

            Some(resource)
        };

        self.create_texture_rgba32_from_resource(None, texture_resource)
    }

    pub fn get_or_create_texture(&mut self, texture_name: &String) -> Result<STextureHandle, &'static str> {

        let uid = {
            let mut s = DefaultHasher::new();
            texture_name.hash(&mut s);
            s.finish()
        };

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.texture_pool.used() {
            if let Some(texture) = &self.texture_pool.get_by_index(i as u16)? {
                if let Some(texture_uid) = texture.uid {
                    if texture_uid == uid {
                        return Ok(self.texture_pool.handle_for_index(i as u16)?);
                    }
                }
            }
        }

        let texture_resource = {
            let mut handle = self.copy_command_list_pool.alloc_list()?;
            let mut copycommandlist = self.copy_command_list_pool.get_list(&handle)?;

            let mut texture_asset = ArrayString::<[_; 128]>::new();
            texture_asset.push_str("assets/");
            texture_asset.push_str(texture_name);
            let (mut _intermediate_resource, mut resource) = n12::load_texture(
                self.device.upgrade().expect("dropped device").deref(),
                copycommandlist.deref_mut(),
                texture_asset.as_str());

            drop(copycommandlist);

            let fenceval = self.copy_command_list_pool.execute_and_free_list(&mut handle)?;
            self.copy_command_list_pool.wait_for_internal_fence_value(fenceval);
            self.copy_command_list_pool.free_allocators();
            assert_eq!(self.copy_command_list_pool.num_free_allocators(), 2);

            unsafe {
                _intermediate_resource.set_debug_name("text inter");
                resource.set_debug_name("text dest");
            }

            Some(resource)
        };

        self.create_texture_rgba32_from_resource(Some(uid), texture_resource)
    }

    pub fn texture_gpu_descriptor(&self, texture: STextureHandle) -> Result<t12::SGPUDescriptorHandle, &'static str> {
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
        is_lit: bool,
    ) -> Result<Self, &'static str> {

        let (models, materials) = tobj::load_obj(&std::path::Path::new(obj_file)).unwrap();
        assert_eq!(models.len(), 1);

        let mesh = mesh_loader.get_or_create_mesh_obj(obj_file, &models[0].mesh);
        let mut diffuse_colour : Vec4 = glm::zero();
        let mut diffuse_texture : Option<STextureHandle> = None;

        if materials.len() > 0 {
            assert_eq!(materials.len(), 1);

            diffuse_colour[0] = materials[0].diffuse[0];
            diffuse_colour[1] = materials[0].diffuse[1];
            diffuse_colour[2] = materials[0].diffuse[2];
            diffuse_colour[3] = 1.0;

            if materials[0].diffuse_texture.len() > 0 {
                diffuse_texture = Some(texture_loader.get_or_create_texture(&materials[0].diffuse_texture)?)
            }
        }

        Ok(Self {
            mesh: mesh?,

            pickable: true,

            // -- material info
            diffuse_colour,
            diffuse_texture,
            diffuse_weight,
            is_lit,
        })
    }

    pub fn new_from_gltf(
        gltf_path: &'static str,
        mesh_loader: &mut SMeshLoader,
        texture_loader: &mut STextureLoader,
        diffuse_weight: f32,
        is_lit: bool,
    ) -> Result<Self, &'static str> {

        let gltf = gltf::Gltf::open(gltf_path).unwrap();

        let mesh = mesh_loader.get_or_create_mesh_gltf(gltf_path, &gltf);

        panic!("not implemented");
    }

    #[allow(dead_code)]
    pub fn set_pickable(&mut self, pickable: bool) {
        self.pickable = pickable;
    }

    /*
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
            is_lit: if self.is_lit { 1.0 } else { 0.0 },
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
    */
}