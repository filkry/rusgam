use allocate::{SAllocatorRef};
use animation::{SAnimHandle, SAnimationLoader, update_joints};
use collections::{SVec};
use entity::{SEntityHandle};
use entity_model;
use game_context::{SGameContext, SFrameContext};
use model::{SModelSkinning, SMeshLoader};

struct SPlayingAnimation {
    animation: SAnimHandle,
    start_time: f32,
}

pub struct SEntityAnimation {
    pub owner: SEntityHandle,

    pub skinning: SModelSkinning,

    playing_animation: Option<SPlayingAnimation>,
}

pub struct SBucket {
    pub instances: SVec<SEntityAnimation>,
}
pub type SHandle = usize;

impl SBucket {
    pub fn new(allocator: &SAllocatorRef, max_entries: usize) -> Result<Self, &'static str> {
        Ok(Self {
            instances: SVec::new(allocator, max_entries, 0)?,
        })
    }

    pub fn add_instance(
        &mut self,
        entity: SEntityHandle,
        model: (&entity_model::SBucket, entity_model::SHandle),
        mesh_loader: &SMeshLoader,
    ) -> Result<SHandle, &'static str> {

        let skinning = {
            let mesh = model.0.get_model(model.1).mesh;
            mesh_loader.bind_skinning(mesh)
        }?;

        self.instances.push(SEntityAnimation{
            owner: entity,
            skinning,
            playing_animation: None,
        });
        Ok(self.instances.len() - 1)
    }

    pub fn purge_entities(&mut self, entities: &[SEntityHandle]) {
        let mut i = 0;
        while i < self.instances.len() {
            let mut purge = false;
            for entity in entities {
                if *entity == self.instances[i].owner {
                    purge = true;
                    break;
                }
            }

            if purge {
                self.instances.swap_remove(i);
            }
            else {
                i = i + 1;
            }
        }
    }

    pub fn play_animation(
        &mut self,
        handle: SHandle,
        anim_loader: &mut SAnimationLoader,
        mesh_loader: &SMeshLoader,
        asset_file_path: &str,
        cur_time_seconds: f32,
    ) {
        let anim_handle = {
            let mesh = self.instances[handle].skinning.mesh;
            let mesh_skinning = mesh_loader.get_mesh_skinning(mesh).unwrap();
            anim_loader.get_or_create_anim(asset_file_path, &mesh_skinning)
        }.unwrap();

        self.instances[handle].playing_animation = Some(SPlayingAnimation{
            animation: anim_handle,
            start_time: cur_time_seconds,
        });
    }

    pub fn update_joints(&mut self, anim_loader: &SAnimationLoader, cur_time_seconds: f32) {
        for instance in self.instances.as_mut() {
            if let Some(pa) = &instance.playing_animation {
                let animation = anim_loader.get_anim(pa.animation).unwrap();
                let anim_time = (cur_time_seconds - pa.start_time) % animation.duration;
                update_joints(
                    &animation,
                    anim_time,
                    &mut instance.skinning.cur_joints_to_parents,
                );
            }
        }
    }

    pub fn handle_for_entity(&self, entity: SEntityHandle) -> Option<SHandle> {
        for (i, instance) in self.instances.as_ref().iter().enumerate() {
            if instance.owner == entity {
                return Some(i);
            }
        }

        None
    }

    pub fn get_skinning(&self, handle: SHandle) -> Result<&SModelSkinning, &'static str> {
        self.instances.get(handle).ok_or("out of bounds").map(|instance| &instance.skinning)
    }

    pub fn get_skinning_for_entity(&self, entity: SEntityHandle) -> Option<&SModelSkinning> {
        let handle_opt = self.handle_for_entity(entity);
        handle_opt.map(|handle| self.get_skinning(handle).expect("handle valid"))
    }
}

pub fn update_animation(game_context: &SGameContext, frame_context: &SFrameContext) {
    game_context.data_bucket.get::<SBucket>()
        .and::<SAnimationLoader>()
        .with_mc(|e_animation, anim_loader| {
            e_animation.update_joints(anim_loader, frame_context.total_time_s);
        });
}

/*
pub fn debug_draw_skeleton(game_context: &SGameContext, frame_context: &SFrameContext) {
    // -- draw skeleton of selected entity
    STACK_ALLOCATOR.with(|sa| {
        data_bucket.get::<render::SRender>().unwrap()
            .and::<entity_model::SBucket>(&data_bucket).unwrap()
            .and::<SEntityBucket>(&data_bucket).unwrap()
            .with_mcc(|render: &mut render::SRender, em: &entity_model::SBucket, entities: &SEntityBucket| {
                if let Some(e) = editmode_ctxt.editing_entity() {
                    let loc = entities.get_entity_location(e);
                    let model_handle = em.handle_for_entity(e).unwrap();
                    let model = em.get_model(model_handle);

                    let mut joint_locs = SVec::new(sa, 128, 0).unwrap();

                    if let Some(bind_joints) = render.mesh_loader().get_mesh_bind_joints(model.mesh) {
                        if let Some(model_skinning) = entities.get_model_skinning(e) {
                            for (ji, joint) in bind_joints.as_ref().iter().enumerate() {
                                let mut local_to_root = model_skinning.cur_joints_to_parents[ji];
                                let mut next_idx_opt = joint.parent_idx;
                                while let Some(next_idx) = next_idx_opt {
                                    local_to_root = STransform::mul_transform(&bind_joints[next_idx].local_to_parent, &local_to_root);
                                    next_idx_opt = bind_joints[next_idx].parent_idx;
                                }

                                let local_to_world = STransform::mul_transform(&loc, &local_to_root);
                                joint_locs.push(local_to_world);
                            }
                        }
                    }

                    for joint_loc in joint_locs.as_ref() {
                        let end = joint_loc.t + glm::quat_rotate_vec3(&joint_loc.r, &Vec3::new(0.0, 1.0, 0.0));
                        render.temp().draw_line(&joint_loc.t, &end, &Vec4::new(0.0, 1.0, 0.0, 1.0), true, None);
                    }
                }
            });
    });
}
*/
