use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Weak;

use glm::{Vec4, Vec3, Vec2, Mat4};
use arrayvec::{ArrayString};
use gltf;

use t12;
use n12;
use n12::descriptorallocator::{descriptor_alloc};
use allocate::{SYSTEM_ALLOCATOR, STACK_ALLOCATOR};
use collections;
use collections::{SStoragePool, SVec};
use safewindows;
use render::shaderbindings;
use rustywindows;
use utils;
use utils::{STransform, gltf_accessor_slice, SHashedStr, hash_str};

#[derive(Debug)]
pub struct SJoint {
    pub local_to_parent: STransform,
    pub parent_idx: Option<usize>,
    pub name: SHashedStr,
}

pub struct SMeshSkinning {
    _vertex_skinning_data: SVec<shaderbindings::SVertexSkinningData>,
    pub vertex_skinning_buffer_resource: n12::SBufferResource<shaderbindings::SVertexSkinningData>,
    pub vertex_skinning_buffer_view: n12::SDescriptorAllocatorAllocation,

    bind_joints: SVec<SJoint>,
    bind_model_to_joint_xforms: SVec<Mat4>,
}

#[allow(dead_code)]
pub struct SMesh {
    uid: u64,

    local_verts: SVec<Vec3>,
    local_normals: SVec<Vec3>,
    uvs: SVec<Vec2>,
    pub(super) indices: SVec<u16>,

    local_aabb: utils::SAABB,

    // -- resources
    pub(super) local_verts_resource: n12::SBufferResource<Vec3>,
    pub(super) local_normals_resource: n12::SBufferResource<Vec3>,
    pub(super) uvs_resource: n12::SBufferResource<Vec2>,
    pub(super) indices_resource: n12::SBufferResource<u16>,

    // -- views
    pub(super) local_verts_vbv: t12::SVertexBufferView,
    pub(super) local_normals_vbv: t12::SVertexBufferView,
    pub(super) uvs_vbv: t12::SVertexBufferView,
    pub(super) indices_ibv: t12::SIndexBufferView,

    // -- SRV descriptors
    srv_descriptors: n12::SDescriptorAllocatorAllocation,

    skinning: Option<SMeshSkinning>, // $$$FRK(TODO): most meshes won't have skinning, this should be factored out
}

pub struct STexture {
    uid: Option<u64>, // if the texture is unique, it will have no ID

    #[allow(dead_code)] // maybe unnecessary?
    //pub(super) srv_heap: &'a n12::descriptorallocator::SDescriptorAllocator,
    pub(super) _diffuse_texture_resource: Option<n12::SResource>,
    pub(super) diffuse_texture_srv: Option<n12::descriptorallocator::SDescriptorAllocatorAllocation>,
}

pub struct SMeshLoader {
    device: Weak<n12::SDevice>,
    copy_command_list_pool: n12::SCommandListPool,
    direct_command_list_pool: n12::SCommandListPool,
    cbv_srv_uav_heap: Weak<n12::descriptorallocator::SDescriptorAllocator>,

    mesh_pool: SStoragePool<SMesh, u16, u16>,
}
pub type SMeshHandle = collections::SPoolHandle<u16, u16>;

pub struct STextureLoader {
    device: Weak<n12::SDevice>,
    copy_command_list_pool: n12::SCommandListPool,
    direct_command_list_pool: n12::SCommandListPool,
    cbv_srv_uav_heap: Weak<n12::descriptorallocator::SDescriptorAllocator>,

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

pub struct SModelSkinning {
    pub mesh: SMeshHandle,

    pub cur_joints_to_parents: SVec<STransform>,

    pub joints_bind_to_cur_resource: n12::SBufferResource<Mat4>,

    pub skinned_verts_resource: n12::SBufferResource<Vec3>,
    pub skinned_verts_vbv: t12::SVertexBufferView,
    pub skinned_normals_resource: n12::SBufferResource<Vec3>,
    pub skinned_normals_vbv: t12::SVertexBufferView,
}

impl SMeshLoader {
    pub fn new(
        device: Weak<n12::SDevice>,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        direct_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        cbv_srv_uav_heap: Weak<n12::SDescriptorAllocator>,
        max_mesh_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device: device.clone(),
            copy_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("bad device").deref(), copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            direct_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("bad device").deref(), direct_command_queue, &winapi.rawwinapi(), 1, 2)?,
            cbv_srv_uav_heap,
            mesh_pool: SStoragePool::create(max_mesh_count),
        })
    }

    pub fn shutdown(&mut self) {
        self.mesh_pool.clear();
    }

    fn sync_create_and_upload_buffer_resource<T>(
        &mut self,
        data: &[T],
        resource_flags: t12::SResourceFlags,
        target_state: t12::EResourceStates,
    ) -> Result<n12::SBufferResource<T>, &'static str> {
        let mut handle = self.copy_command_list_pool.alloc_list()?;
        let mut copycommandlist = self.copy_command_list_pool.get_list(&handle)?;

        let resource = {
            copycommandlist.update_buffer_resource(
                self.device.upgrade().expect("device dropped").deref(), data, resource_flags,
            )?
        };
        drop(copycommandlist);

        let fence_val = self.copy_command_list_pool.execute_and_free_list(&mut handle)?;
        drop(handle);

        // --  This is the sync part - waiting so we can drop intermediate resource safely
        self.copy_command_list_pool.wait_for_internal_fence_value(fence_val);

        // -- have the direct queue wait on the copy upload to complete
        // -- shouldn't be necessary in sync version
        /*
        self.direct_command_list_pool.gpu_wait(
            self.copy_command_list_pool.get_internal_fence(),
            fence_val,
        )?;
        */

        // -- transition resources
        let mut handle  = self.direct_command_list_pool.alloc_list()?;
        let mut direct_command_list = self.direct_command_list_pool.get_list(&handle)?;

        direct_command_list.transition_resource(
            &resource.destinationresource.raw,
            t12::EResourceStates::CopyDest,
            target_state,
        )?;

        drop(direct_command_list);
        self.direct_command_list_pool.execute_and_free_list(&mut handle)?;

        Ok(resource.destinationresource)
    }

    fn create_mesh_descriptors(
        &mut self,
        verts_resource: &n12::SBufferResource<Vec3>,
        norms_resource: &n12::SBufferResource<Vec3>,
    ) -> Result<n12::SDescriptorAllocatorAllocation, &'static str> {
        // -- create srv_views
        let descriptors = descriptor_alloc(&self.cbv_srv_uav_heap.upgrade().expect("allocator dropped"), 2)?;
        let vert_srv_desc = verts_resource.create_srv_desc();
        let norm_srv_desc = norms_resource.create_srv_desc();

        self.device.upgrade().expect("device dropped").create_shader_resource_view(
            &verts_resource.raw,
            &vert_srv_desc,
            descriptors.cpu_descriptor(SMesh::SRVS_VERT_IDX),
        )?;
        self.device.upgrade().expect("device dropped").create_shader_resource_view(
            &norms_resource.raw,
            &norm_srv_desc,
            descriptors.cpu_descriptor(SMesh::SRVS_NORM_IDX),
        )?;

        Ok(descriptors)
    }

    pub fn get_or_create_mesh_gltf(&mut self, asset_file_path: &'static str, gltf_data: &gltf::Gltf) -> Result<SMeshHandle, &'static str> {
        let uid = hash_str(asset_file_path);

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

        let positions : &[Vec3] = gltf_accessor_slice(
            &primitive.get(&gltf::mesh::Semantic::Positions).unwrap(),
            gltf::accessor::DataType::F32,
            gltf::accessor::Dimensions::Vec3,
            &buffer_bytes,
        );
        //println!("Dumped GLTF positions: {:?}", positions);
        let normals : &[Vec3] = gltf_accessor_slice(
            &primitive.get(&gltf::mesh::Semantic::Normals).unwrap(),
            gltf::accessor::DataType::F32,
            gltf::accessor::Dimensions::Vec3,
            &buffer_bytes,
        );
        //println!("Dumped GLTF normals: {:?}", normals);
        let bin_uvs : &[Vec2] = gltf_accessor_slice(
            &primitive.get(&gltf::mesh::Semantic::TexCoords(0)).unwrap(),
            gltf::accessor::DataType::F32,
            gltf::accessor::Dimensions::Vec2,
            &buffer_bytes,
        );

        assert!(positions.len() == normals.len());
        assert!(positions.len() == bin_uvs.len());

        let allocator = SYSTEM_ALLOCATOR();

        let local_verts = SVec::<Vec3>::new_copy_slice(&allocator, positions).unwrap();
        let local_normals = SVec::<Vec3>::new_copy_slice(&allocator, normals).unwrap();
        let uvs = SVec::<Vec2>::new_copy_slice(&allocator, bin_uvs).unwrap();

        let indices_bin : &[u16] = gltf_accessor_slice(
            &primitive.indices().unwrap(),
            gltf::accessor::DataType::U16,
            gltf::accessor::Dimensions::Scalar,
            &buffer_bytes,
        );

        let indices = SVec::<u16>::new_copy_slice(&allocator, indices_bin).unwrap();

        let local_verts_resource = self.sync_create_and_upload_buffer_resource(
            local_verts.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let local_verts_vbv = local_verts_resource.raw.create_vertex_buffer_view()?;

        let local_normals_resource = self.sync_create_and_upload_buffer_resource(
            local_normals.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let local_normals_vbv = local_normals_resource.raw.create_vertex_buffer_view()?;

        let uvs_resource = self.sync_create_and_upload_buffer_resource(
            uvs.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let uvs_vbv = uvs_resource.raw.create_vertex_buffer_view()?;

        let indices_resource = self.sync_create_and_upload_buffer_resource(
            indices.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::IndexBuffer,
        )?;
        let indices_ibv = indices_resource.raw.create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

        let local_aabb = utils::SAABB::new_from_points(local_verts.as_slice());
        //println!("Asset name: {}\nAABB: {:?}", asset_name, local_aabb);

        let srv_descriptors = self.create_mesh_descriptors(&local_verts_resource, &local_normals_resource)?;

        // -- load skeleton data

        let mut skinning = None;
        let joints_accessor_opt = &primitive.get(&gltf::mesh::Semantic::Joints(0));
        if let Some(joints_accessor) = joints_accessor_opt {
            let joints : &[[u16; 4]] = gltf_accessor_slice(
                joints_accessor,
                gltf::accessor::DataType::U16,
                gltf::accessor::Dimensions::Vec4,
                &buffer_bytes,
            );
            let weights : &[[f32; 4]] = gltf_accessor_slice(
                &primitive.get(&gltf::mesh::Semantic::Weights(0)).unwrap(),
                gltf::accessor::DataType::F32,
                gltf::accessor::Dimensions::Vec4,
                &buffer_bytes,
            );
            assert!(joints.len() == weights.len());

            let mut vertex_skinning_data = SVec::<shaderbindings::SVertexSkinningData>::new(&allocator, joints.len(), 0).unwrap();
            for i in 0..joints.len() {
                vertex_skinning_data.push(shaderbindings::SVertexSkinningData{
                    joints: [joints[i][0] as u32, joints[i][1] as u32, joints[i][2] as u32, joints[i][3] as u32],
                    joint_weights: weights[i],
                });
            }

            let vertex_skinning_buffer_resource = self.sync_create_and_upload_buffer_resource(
                vertex_skinning_data.as_slice(),
                t12::SResourceFlags::from(t12::EResourceFlags::ENone),
                t12::EResourceStates::NonPixelShaderResource,
            )?;
            let vertex_skinning_buffer_view = {
                let descriptors = descriptor_alloc(&self.cbv_srv_uav_heap.upgrade().expect("allocator dropped"), 1)?;
                let srv_desc = vertex_skinning_buffer_resource.create_srv_desc();
                self.device.upgrade().expect("device dropped").create_shader_resource_view(
                    &vertex_skinning_buffer_resource.raw,
                    &srv_desc,
                    descriptors.cpu_descriptor(0),
                )?;

                descriptors
            };

            assert!(gltf_data.skins().len() == 1, "Can't handle multi-skin model currently");
            let skin = gltf_data.skins().nth(0).unwrap();

            let inverse_bind_matrices_bin : &[Mat4] = gltf_accessor_slice(
                &skin.inverse_bind_matrices().unwrap(),
                gltf::accessor::DataType::F32,
                gltf::accessor::Dimensions::Mat4,
                &buffer_bytes,
            );
            let bind_model_to_joint_xforms = SVec::<Mat4>::new_copy_slice(&allocator, inverse_bind_matrices_bin).unwrap();

            let bind_joints = STACK_ALLOCATOR.with(|sa| {
                let mut result = SVec::<SJoint>::new(&allocator, skin.joints().count(), 0).unwrap();

                let mut index_map = SVec::<Option<usize>>::new(&sa.as_ref(), gltf_data.nodes().count(), 0).unwrap();
                for _ in 0..index_map.capacity() {
                    index_map.push(None);
                }

                // -- first pass just create all the transforms
                for joint_node in skin.joints() {
                    let (trans, rot, scale) = joint_node.transform().decomposed();
                    assert!(scale[0] == scale[1]  && scale[0] == scale[2]);
                    let transform = STransform::new(
                        &glm::vec3(trans[0], trans[1], trans[2]),
                        &glm::quat(rot[0], rot[1], rot[2], rot[3]),
                        scale[0],
                    );

                    result.push(SJoint{
                        local_to_parent: transform,
                        parent_idx: None,
                        name: hash_str(joint_node.name().unwrap()),
                    });
                    index_map[joint_node.index()] = Some(result.len() - 1);
                }

                // -- second pass set up parent relationships
                for joint_node in skin.joints() {
                    let result_idx = index_map[joint_node.index()].unwrap();

                    for child_node in joint_node.children() {
                        if let Some(child_result_idx) = index_map[child_node.index()] {
                            result[child_result_idx].parent_idx = Some(result_idx);
                        }
                    }
                }

                assert!(result[0].parent_idx == None);
                for result_idx in 1..result.len() {
                    assert!(result[result_idx].parent_idx.is_some());
                }

                result
            });

            /*
            for i in 0..bind_joints.len() {
                println!("bind_joints[{:?}]: {:?}", i, bind_joints[i]);
            }
            */

            skinning = Some(SMeshSkinning{
                _vertex_skinning_data: vertex_skinning_data,
                vertex_skinning_buffer_resource,
                vertex_skinning_buffer_view,

                bind_joints,
                bind_model_to_joint_xforms,
            })
        }

        let mesh = SMesh{
            uid: uid,

            local_verts,
            local_normals,
            uvs,
            indices,

            local_aabb,

            local_verts_resource,
            local_normals_resource,
            uvs_resource,
            indices_resource,

            local_verts_vbv,
            local_normals_vbv,
            uvs_vbv,
            indices_ibv,

            srv_descriptors,

            skinning,
        };

        return self.mesh_pool.insert_val(mesh)
    }

    pub fn get_or_create_mesh_obj(&mut self, asset_name: &'static str, tobj_mesh: &tobj::Mesh) -> Result<SMeshHandle, &'static str> {
        let uid = hash_str(asset_name);

        // -- $$$FRK(TODO): replace with some accelerated lookup structure
        for i in 0..self.mesh_pool.used() {
            if let Some(mesh) = &self.mesh_pool.get_by_index(i as u16).unwrap() {
                if mesh.uid == uid {
                    return Ok(self.mesh_pool.handle_for_index(i as u16)?);
                }
            }
        }

        let allocator = SYSTEM_ALLOCATOR();

        assert!(tobj_mesh.positions.len() % 3 == 0);
        assert!(tobj_mesh.texcoords.len() / 2 == tobj_mesh.positions.len() / 3);
        assert!(tobj_mesh.normals.len() == tobj_mesh.positions.len());

        fn to_memvec<I, T>(input: &[I]) -> SVec<T> {
            let (_a, input_aligned, _b) = unsafe { input.align_to::<T>() };
            assert!(_a.len() == 0 && _b.len() == 0);
            SVec::<T>::new_copy_slice(&SYSTEM_ALLOCATOR(), input_aligned).unwrap()
        }

        let local_verts : SVec::<Vec3> = to_memvec(&tobj_mesh.positions);
        let local_normals : SVec::<Vec3> = to_memvec(&tobj_mesh.normals);
        let uvs : SVec::<Vec2> = to_memvec(&tobj_mesh.texcoords);

        let mut indices : SVec::<u16> = SVec::new(&allocator, tobj_mesh.indices.len(), 0)?;
        for index in &tobj_mesh.indices {
            indices.push(*index as u16);
        }

        drop(tobj_mesh);

        let local_verts_resource = self.sync_create_and_upload_buffer_resource(
            local_verts.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let local_verts_vbv = local_verts_resource.raw.create_vertex_buffer_view()?;

        let local_normals_resource = self.sync_create_and_upload_buffer_resource(
            local_normals.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let local_normals_vbv = local_normals_resource.raw.create_vertex_buffer_view()?;

        let uvs_resource = self.sync_create_and_upload_buffer_resource(
            uvs.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::VertexAndConstantBuffer,
        )?;
        let uvs_vbv = uvs_resource.raw.create_vertex_buffer_view()?;

        let indices_resource = self.sync_create_and_upload_buffer_resource(
            indices.as_slice(),
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::IndexBuffer,
        )?;
        let indices_ibv = indices_resource.raw.create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

        let srv_descriptors = self.create_mesh_descriptors(&local_verts_resource, &local_normals_resource)?;

        let local_aabb = utils::SAABB::new_from_points(local_verts.as_slice());

        let mesh = SMesh{
            uid: uid,

            local_verts,
            local_normals,
            uvs,
            indices,

            local_aabb,

            local_verts_resource,
            local_normals_resource,
            uvs_resource,
            indices_resource,

            local_verts_vbv,
            local_normals_vbv,
            uvs_vbv,
            indices_ibv,

            srv_descriptors,

            skinning: None,
        };

        return self.mesh_pool.insert_val(mesh)
    }

    pub fn bind_skinning(
        &self,
        mesh: SMeshHandle,
    ) -> Result<SModelSkinning, &'static str> {
        let bind_joints = self.get_mesh_bind_joints(mesh).unwrap();

        let mut joints_bind_to_cur_resource = self.device.upgrade().expect("device dropped").create_committed_buffer_resource_for_type::<Mat4>(
            t12::EHeapType::Upload,
            t12::SResourceFlags::from(t12::EResourceFlags::ENone),
            t12::EResourceStates::GenericRead,
            bind_joints.len(),
        )?;

        let allocator = SYSTEM_ALLOCATOR();

        joints_bind_to_cur_resource.map();
        //frame_model_to_joint_to_world_xforms_resource.copy_to_map(bind_joints.as_ref());

        let mut cur_joints_to_parents = SVec::<STransform>::new(&allocator, bind_joints.len(), 0)?;
        for joint in bind_joints.as_ref() {
            cur_joints_to_parents.push(joint.local_to_parent);
        }

        cur_joints_to_parents[1].t = Vec3::new(1.0, 0.0, 0.0);

        // -- $$$FRK(TODO, HACK): lazily working around the borrow checker here
        let initial_verts = SVec::<Vec3>::new_copy_slice(&allocator, self.get_mesh_local_vertices(mesh))?;
        let initial_normals = SVec::<Vec3>::new_copy_slice(&allocator, self.get_mesh_local_normals(mesh))?;

        let skinned_verts_resource = self.device.upgrade().expect("device dropped").create_committed_buffer_resource_for_data(
            t12::EHeapType::Default,
            t12::SResourceFlags::from(t12::EResourceFlags::AllowUnorderedAccess),
            t12::EResourceStates::UnorderedAccess,
            initial_verts.as_ref(),
        )?;
        let skinned_verts_vbv = skinned_verts_resource.raw.create_vertex_buffer_view()?;

        let skinned_normals_resource = self.device.upgrade().expect("device dropped").create_committed_buffer_resource_for_data(
            t12::EHeapType::Default,
            t12::SResourceFlags::from(t12::EResourceFlags::AllowUnorderedAccess),
            t12::EResourceStates::UnorderedAccess,
            initial_normals.as_ref(),
        )?;
        let skinned_normals_vbv = skinned_normals_resource.raw.create_vertex_buffer_view()?;

        Ok(SModelSkinning{
            mesh,
            cur_joints_to_parents,
            joints_bind_to_cur_resource,

            skinned_verts_resource,
            skinned_verts_vbv,
            skinned_normals_resource,
            skinned_normals_vbv,
        })
    }

    pub fn get_mesh_local_aabb(&self, mesh: SMeshHandle) -> &utils::SAABB {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        &mesh.local_aabb
    }

    pub fn get_mesh_local_vertices(&self, mesh: SMeshHandle) -> &SVec<Vec3> {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        &mesh.local_verts
    }

    pub fn get_mesh_local_normals(&self, mesh: SMeshHandle) -> &SVec<Vec3> {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        &mesh.local_normals
    }

    pub fn get_mesh_skinning(&self, mesh: SMeshHandle) -> Option<&SMeshSkinning> {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        mesh.skinning.as_ref()
    }

    pub fn get_mesh_bind_joints(&self, mesh: SMeshHandle) -> Option<&SVec<SJoint>> {
        let mesh = self.mesh_pool.get(mesh).unwrap();
        if let Some(skinning) = &mesh.skinning {
            Some(&skinning.bind_joints)
        }
        else {
            None
        }
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

        break_assert!(mesh.indices.len() % 3 == 0);
        let num_tris = mesh.indices.len() / 3;

        let mut min_t = None;

        for ti in 0..num_tris {
            let ti_vi_0 = mesh.indices[ti * 3 + 0];
            let ti_vi_1 = mesh.indices[ti * 3 + 1];
            let ti_vi_2 = mesh.indices[ti * 3 + 2];

            let v0_pos = &mesh.local_verts[ti_vi_0 as usize];
            let v1_pos = &mesh.local_verts[ti_vi_1 as usize];
            let v2_pos = &mesh.local_verts[ti_vi_2 as usize];

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

    pub fn vertex_count(&self, mesh_handle: SMeshHandle) -> usize {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        mesh.local_verts.len()
    }

    pub fn index_count(&self, mesh_handle: SMeshHandle) -> usize {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        mesh.indices.len()
    }

    pub fn local_verts_resource(&self, mesh_handle: SMeshHandle) -> &n12::SBufferResource<Vec3> {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.local_verts_resource
    }

    pub fn local_normals_resource(&self, mesh_handle: SMeshHandle) -> &n12::SBufferResource<Vec3> {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.local_normals_resource
    }

    pub fn local_verts_vbv(&self, mesh_handle: SMeshHandle) -> &t12::SVertexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.local_verts_vbv
    }

    pub fn local_normals_vbv(&self, mesh_handle: SMeshHandle) -> &t12::SVertexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.local_normals_vbv
    }

    pub fn uvs_vbv(&self, mesh_handle: SMeshHandle) -> &t12::SVertexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.uvs_vbv
    }

    pub fn indices_ibv(&self, mesh_handle: SMeshHandle) -> &t12::SIndexBufferView {
        let mesh = self.mesh_pool.get(mesh_handle).expect("querying invalid mesh");
        &mesh.indices_ibv
    }

    pub fn set_index_buffer_and_draw(
        &self,
        mesh_handle: SMeshHandle,
        cl: &mut n12::SCommandList,
    ) -> Result<(), &'static str> {
        let mesh = self.mesh_pool.get(mesh_handle)?;

        /*
        */
        cl.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
        cl.ia_set_index_buffer(&mesh.indices_ibv);
        cl.draw_indexed_instanced(mesh.indices.len() as u32, 1, 0, 0, 0);

        Ok(())
    }
}

impl STextureLoader {
    pub fn new(
        device: Weak<n12::SDevice>,
        winapi: &rustywindows::SWinAPI,
        copy_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        direct_command_queue: Weak<RefCell<n12::SCommandQueue>>,
        cbv_srv_uav_heap: Weak<n12::SDescriptorAllocator>,
        max_texture_count: u16,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            device: device.clone(),
            copy_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("dropped device").deref(), copy_command_queue, &winapi.rawwinapi(), 1, 2)?,
            direct_command_list_pool: n12::SCommandListPool::create(device.upgrade().expect("dropped device").deref(), direct_command_queue, &winapi.rawwinapi(), 1, 10)?,
            cbv_srv_uav_heap,

            texture_pool: SStoragePool::create(max_texture_count),
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

            let descriptors = descriptor_alloc(&self.cbv_srv_uav_heap.upgrade().expect("allocator dropped"), 1)?;
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

        let uid = hash_str(texture_name);

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

impl SMeshSkinning {
    pub fn joint_index_by_name(&self, name: &str) -> Option<usize> {
        let hashed_name = utils::hash_str(name);
        for (ji, joint) in self.bind_joints.as_ref().iter().enumerate() {
            if joint.name == hashed_name {
                return Some(ji);
            }
        }

        None
    }
}

impl SMesh {
    const SRVS_VERT_IDX: usize = 0;
    const SRVS_NORM_IDX: usize = 1;
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
        _texture_loader: &mut STextureLoader,
        diffuse_weight: f32,
        is_lit: bool,
    ) -> Result<Self, &'static str> {

        let gltf = gltf::Gltf::open(gltf_path).unwrap();

        let mesh = mesh_loader.get_or_create_mesh_gltf(gltf_path, &gltf);

        let diffuse_colour = Vec4::new(0.7, 0.0, 0.3, 1.0);
        let diffuse_texture : Option<STextureHandle> = None;

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

    #[allow(dead_code)]
    pub fn set_pickable(&mut self, pickable: bool) {
        self.pickable = pickable;
    }

}

impl SModelSkinning {
    pub fn update_skinning_joint_buffer(&mut self, mesh_loader: &SMeshLoader) {
        STACK_ALLOCATOR.with(|sa| {
            if let Some(skinning) = mesh_loader.get_mesh_skinning(self.mesh) {
                let bind_joints = mesh_loader.get_mesh_bind_joints(self.mesh).unwrap();

                let mut frame_joint_to_model = SVec::<STransform>::new(&sa.as_ref(), skinning.bind_joints.len(), 0).unwrap();
                // -- we can just iterate the table once because all parents are earlier in the table,
                // -- this is essentially flattening the table from joint -> parent to joint -> model
                for i in 0..bind_joints.len() {
                    let local_to_model = {
                        if let Some(parent_idx) = bind_joints[i].parent_idx {
                            assert!(parent_idx < i);
                            STransform::mul_transform(&frame_joint_to_model[parent_idx], &self.cur_joints_to_parents[i])
                        }
                        else {
                            self.cur_joints_to_parents[i]
                        }
                    };

                    frame_joint_to_model.push(local_to_model);
                }

                let mut frame_joints_bind_to_cur = SVec::<Mat4>::new(&sa.as_ref(), skinning.bind_joints.len(), 0).unwrap();
                // -- essentially the pipeline for this is:
                // -- (bind vertex to bind joint) x (frame joint to model) x (model to world)

                for i in 0..bind_joints.len() {
                    frame_joints_bind_to_cur.push(frame_joint_to_model[i].as_mat4() * skinning.bind_model_to_joint_xforms[i]);
                }

                self.joints_bind_to_cur_resource.copy_to_map(frame_joints_bind_to_cur.as_ref());
            }
        });
    }
}