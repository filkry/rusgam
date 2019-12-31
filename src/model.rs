use std::cell::RefCell;

use glm::{Vec3, Vec2, Mat4};
use arrayvec::{ArrayString};

use t12;
use n12;
use allocate::{SMemVec, SYSTEM_ALLOCATOR};

#[allow(dead_code)]
struct SVertexPosColourUV {
    position: Vec3,
    colour: Vec3,
    normal: Vec3,
    uv: Vec2,
}

#[allow(dead_code)]
pub struct SModel<'a> {
    per_vertex_data: SMemVec<'a, SVertexPosColourUV>,
    pub(super) triangle_indices: SMemVec<'a, u16>,

    pub(super) vertex_buffer_resource: n12::SResource,
    pub(super) vertex_buffer_view: t12::SVertexBufferView,
    pub(super) index_buffer_resource: n12::SResource,
    pub(super) index_buffer_view: t12::SIndexBufferView,

    pub(super) srv_heap: &'a RefCell<n12::descriptorallocator::SDescriptorAllocator>,
    pub(super) diffuse_texture_resource: Option<n12::SResource>,
    pub(super) diffuse_texture_srv: Option<n12::descriptorallocator::SDescriptorAllocatorAllocation>,
}

impl<'a> SModel<'a> {

    pub fn new_from_obj(
        obj_file: &'static str,
        device: &n12::SDevice,
        copy_command_pool: &mut n12::SCommandListPool,
        direct_command_pool: &mut n12::SCommandListPool,
        srv_heap: &'a RefCell<n12::descriptorallocator::SDescriptorAllocator>,
    ) -> Result<Self, &'static str> {

        let (models, materials) = tobj::load_obj(&std::path::Path::new(obj_file)).unwrap();
        assert_eq!(models.len(), 1);

        let mut vert_vec = SMemVec::<SVertexPosColourUV>::new(&SYSTEM_ALLOCATOR, models[0].mesh.positions.len(), 0).unwrap();
        let mut index_vec = SMemVec::<u16>::new(&SYSTEM_ALLOCATOR, models[0].mesh.indices.len(), 0).unwrap();

        let mut diffuse = Vec3::new(1.0, 0.0, 1.0);
        if materials.len() > 0 {
            assert_eq!(materials.len(), 1);
            let material = &materials[0];
            diffuse[0] = material.diffuse[0];
            diffuse[1] = material.diffuse[1];
            diffuse[2] = material.diffuse[2];
        }

        for model in models {
            assert!(model.mesh.positions.len() % 3 == 0);
            assert!(model.mesh.texcoords.len() / 2 == model.mesh.positions.len() / 3);
            assert!(model.mesh.normals.len() == model.mesh.positions.len());

            for vidx in 0..model.mesh.positions.len() / 3 {
                vert_vec.push(SVertexPosColourUV {
                    position: Vec3::new(
                        model.mesh.positions[vidx * 3],
                        model.mesh.positions[vidx * 3 + 1],
                        model.mesh.positions[vidx * 3 + 2],
                    ),
                    colour: diffuse,
                    uv: Vec2::new(
                        model.mesh.texcoords[vidx * 2],
                        model.mesh.texcoords[vidx * 2 + 1],
                    ),
                    normal: Vec3::new(
                        model.mesh.normals[vidx * 3],
                        model.mesh.normals[vidx * 3 + 1],
                        model.mesh.normals[vidx * 3 + 2],
                    ),
                });
            }

            for idx in model.mesh.indices {
                index_vec.push(idx as u16);
            }
        }

        // -- generate vertex/index resources and views
        let (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview) = {
            let handle = copy_command_pool.alloc_list()?;
            let copycommandlist = copy_command_pool.get_list(handle)?;

            let mut vertbufferresource = {
                let vertbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(&device, vert_vec.as_slice(), vertbufferflags)?
            };
            let vertexbufferview = vertbufferresource
                .destinationresource
                .create_vertex_buffer_view()?;

            let mut indexbufferresource = {
                let indexbufferflags = t12::SResourceFlags::from(t12::EResourceFlags::ENone);
                copycommandlist.update_buffer_resource(&device, index_vec.as_slice(), indexbufferflags)?
            };
            let indexbufferview = indexbufferresource
                .destinationresource
                .create_index_buffer_view(t12::EDXGIFormat::R16UINT)?;

            let fenceval = copy_command_pool.execute_and_free_list(handle)?;
            copy_command_pool.wait_for_internal_fence_value(fenceval);
            copy_command_pool.free_allocators();
            assert_eq!(copy_command_pool.num_free_allocators(), 2);

            // -- debug
            unsafe {
                vertbufferresource.destinationresource.set_debug_name("vert dest");
                vertbufferresource.intermediateresource.set_debug_name("vert inter");
                indexbufferresource.destinationresource.set_debug_name("index dest");
                indexbufferresource.intermediateresource.set_debug_name("index inter");
            }

            (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
        };

        let mut diffuse_texture_resource = None;
        let mut diffuse_texture_srv = None;
        if materials.len() > 0 {
            assert_eq!(materials.len(), 1);

            let material = &materials[0];

            // -- load texture
            if material.diffuse_texture.len() > 0 {
                // -- load texture resource
                diffuse_texture_resource = {
                    let handle = copy_command_pool.alloc_list()?;
                    let copycommandlist = copy_command_pool.get_list(handle)?;

                    let mut texture_asset = ArrayString::<[_; 128]>::new();
                    texture_asset.push_str("assets/");
                    texture_asset.push_str(&materials[0].diffuse_texture);
                    let (mut _intermediate_resource, mut resource) = n12::load_texture(&device, copycommandlist, texture_asset.as_str());

                    let fenceval = copy_command_pool.execute_and_free_list(handle)?;
                    copy_command_pool.wait_for_internal_fence_value(fenceval);
                    copy_command_pool.free_allocators();
                    assert_eq!(copy_command_pool.num_free_allocators(), 2);

                    unsafe {
                        _intermediate_resource.set_debug_name("text inter");
                        resource.set_debug_name("text dest");
                    }

                    Some(resource)
                };

                // -- transition texture to PixelShaderResource
                {
                    let handle = direct_command_pool.alloc_list()?;
                    let list = direct_command_pool.get_list(handle)?;

                    list.transition_resource(
                        &diffuse_texture_resource.as_ref().unwrap(),
                        t12::EResourceStates::CopyDest,
                        t12::EResourceStates::PixelShaderResource,
                    )
                    .unwrap();

                    let fenceval = direct_command_pool.execute_and_free_list(handle)?;
                    direct_command_pool.wait_for_internal_fence_value(fenceval);
                }

                // -- get texture SRV
                diffuse_texture_srv = {
                    let srv_desc = t12::SShaderResourceViewDesc {
                        format: t12::EDXGIFormat::R8G8B8A8UNorm,
                        view: t12::ESRV::Texture2D {
                            data: t12::STex2DSRV {
                                mip_levels: 1,
                                ..Default::default()
                            },
                        },
                    };

                    let descriptors = srv_heap.borrow_mut().alloc(1)?;
                    device.create_shader_resource_view(
                        diffuse_texture_resource.as_ref().unwrap(),
                        &srv_desc,
                        descriptors.cpu_descriptor(0),
                    )?;

                    Some(descriptors)
                };
            }
        }

        Ok(Self {
            per_vertex_data: vert_vec,
            triangle_indices: index_vec,

            vertex_buffer_resource: vertbufferresource.destinationresource,
            vertex_buffer_view: vertexbufferview,
            index_buffer_resource: indexbufferresource.destinationresource,
            index_buffer_view: indexbufferview,

            srv_heap: srv_heap,
            diffuse_texture_resource: diffuse_texture_resource,
            diffuse_texture_srv: diffuse_texture_srv,
        })
    }

    pub fn render(&self, cl: &mut n12::SCommandList, view_projection: &glm::Mat4, model_matrix: &glm::Mat4) {
        // -- assuming the same pipline state, root signature, viewport, scissor rect,
        // -- render target, for every model for now. These are set
        // -- outside of here

        // -- setup input assembler
        cl.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
        cl.ia_set_vertex_buffers(0, &[&self.vertex_buffer_view]);
        cl.ia_set_index_buffer(&self.index_buffer_view);

        cl.set_descriptor_heaps(&[&self.srv_heap.borrow().raw_heap()]);
        if let Some(dts) = &self.diffuse_texture_srv {
            cl.set_graphics_root_32_bit_constants(1, &1.0f32, 0);
            cl.set_graphics_root_descriptor_table(2, &dts.gpu_descriptor(0));
        }
        else {
            cl.set_graphics_root_32_bit_constants(1, &0.0f32, 0);
        }

        #[allow(dead_code)]
        struct SModelViewProjection {
            model: Mat4,
            view_projection: Mat4,
            mvp: Mat4,
        }

        let mvp_matrix = view_projection * model_matrix;
        let mvp = SModelViewProjection{
            model: model_matrix.clone(),
            view_projection: view_projection.clone(),
            mvp: mvp_matrix,
        };

        cl.set_graphics_root_32_bit_constants(0, &mvp, 0);

        // -- draw
        cl.draw_indexed_instanced(self.triangle_indices.len() as u32, 1, 0, 0, 0);
    }
}

impl <'a> Drop for SModel<'a> {
    fn drop(&mut self) {
        if let Some(dts) = &mut self.diffuse_texture_srv {
            self.srv_heap.borrow_mut().free(dts);
        }
    }
}