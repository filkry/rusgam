use allocate::{SMemVec, TMemAllocator};
use animation::{SAnimHandle, SAnimationLoader, update_joints};
use entity::{SEntityHandle};
use entity_model;
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

pub struct SBucket<'a> {
    pub instances: SMemVec<'a, SEntityAnimation>,
}
pub type SHandle = usize;

impl<'a> SBucket<'a> {
    pub fn new(allocator: &'a dyn TMemAllocator, max_entries: usize) -> Result<Self, &'static str> {
        Ok(Self {
            instances: SMemVec::new(allocator, max_entries, 0)?,
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