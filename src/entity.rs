use allocate::{TMemAllocator, SMemVec};
use model::SModel;
use utils::{STransform, SAABB};
use collections::{SStoragePool, SPoolHandle};
use databucket::{SDataBucket};
use bvh;
use render;

#[allow(dead_code)]
struct SEntity {
    debug_name: Option<&'static str>,
    location: STransform,
    model: Option<SModel>,
    identity_aabb: Option<SAABB>, // $$$FRK(TODO): ONLY putting this in here right now to avoid moving the renderer!
    bvh_entry: bvh::SNodeHandle,
}

#[allow(dead_code)]
pub struct SEntityBucket {
    entities: SStoragePool<SEntity, u16, u16>,
}

pub type SEntityHandle = SPoolHandle<u16, u16>;

impl SEntity {
    pub fn new() -> Self {
        Self {
            debug_name: None,
            location: STransform::default(),
            model: None,
            identity_aabb: None,
            bvh_entry: SPoolHandle::default(),
        }
    }
}

impl SEntityBucket {
    pub fn new(poolid: u64, max_entities: u16) -> Self {
        Self {
            entities: SStoragePool::create(poolid, max_entities),
        }
    }

    pub fn create_entity(&mut self) -> Result<SEntityHandle, &'static str> {
        self.entities.insert_val(SEntity::new())
    }

    pub fn set_entity_debug_name(&mut self, entity: SEntityHandle, debug_name: &'static str) {
        self.entities.get_mut(entity).expect("invalid entity").debug_name = Some(debug_name);
    }

    pub fn get_entity_location(&self, entity: SEntityHandle) -> STransform {
        self.entities.get(entity).expect("invalid entity").location
    }

    pub fn set_entity_location(&mut self, entity: SEntityHandle, location: STransform, data_bucket: &SDataBucket) {
        self.entities.get_mut(entity).expect("invalid entity").location = location;

        if let Some(bvh) = data_bucket.get_bvh() {
            bvh.with_mut(|bvh: &mut bvh::STree| {
                let bvh_entry = self.get_entity_bvh_entry(entity);
                let identity_aabb_opt = self.entities.get(entity).unwrap().identity_aabb;
                if let Some(identity_aabb) = identity_aabb_opt {
                    if bvh_entry.valid() {
                        bvh.remove(bvh_entry);
                        let transformed_aabb = SAABB::transform(&identity_aabb, &location);
                        let entry = bvh.insert(entity, &transformed_aabb);
                        self.set_entity_bvh_entry(entity, entry);
                    }
                }
            });
        }
    }

    pub fn get_entity_model(&self, entity: SEntityHandle) -> Option<SModel> {
        self.entities.get(entity).expect("invalid entity").model
    }

    pub fn set_entity_model(&mut self, entity: SEntityHandle, model: SModel, data_bucket: &SDataBucket) {
        let data = self.entities.get_mut(entity).expect("invalid entity");
        data.model = Some(model);

        data_bucket.get_renderer().unwrap().with(|render: &render::SRender| {
            data.identity_aabb = Some(render.mesh_loader().get_mesh_local_aabb(model.mesh).clone());
        });
    }

    pub fn get_entity_bvh_entry(&self, entity: SEntityHandle) -> bvh::SNodeHandle {
        self.entities.get(entity).expect("invalid entity").bvh_entry
    }

    pub fn set_entity_bvh_entry(&mut self, entity: SEntityHandle, bvh_entry: bvh::SNodeHandle) {
        self.entities.get_mut(entity).expect("invalid entity").bvh_entry = bvh_entry;
    }

    pub fn build_render_data<'a>(&self, allocator: &'a dyn TMemAllocator) -> (SMemVec<'a, SEntityHandle>, SMemVec<'a, STransform>, SMemVec<'a, SModel>) {
        // -- $$$FRK(TODO): if the stack allocator is used, returning these is only safe if the caller makes references to each member  (no _)
        let mut entities = SMemVec::<SEntityHandle>::new(allocator, self.entities.used(), 0).expect("alloc fail");
        let mut transforms = SMemVec::<STransform>::new(allocator, self.entities.used(), 0).expect("alloc fail");
        let mut models = SMemVec::<SModel>::new(allocator, self.entities.used(), 0).expect("alloc fail");

        for entity_idx in 0..self.entities.max() {
            if let Ok(Some(e)) = self.entities.get_by_index(entity_idx) {
                if let Some(m) = e.model {
                    entities.push(self.entities.handle_for_index(entity_idx).unwrap());
                    transforms.push(e.location);
                    models.push(m);
                }
            }
        }

        (entities, transforms, models)
    }

    pub fn show_imgui_window(&mut self, entity: SEntityHandle, imgui_ui: &imgui::Ui) {
        use imgui::*;

        Window::new(im_str!("Selected entity"))
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(&imgui_ui, || {
                if let Some(n) = self.entities.get(entity).expect("invalid entity").debug_name {
                    imgui_ui.text(im_str!("debug_name: {}", n));
                }
                imgui_ui.text(im_str!("index: {}, generation: {}", entity.index(), entity.generation()));
                imgui_ui.separator();
                let mut pos = {
                    let entity = self.entities.get(entity).expect("invalid entity");
                    [entity.location.t.x, entity.location.t.y, entity.location.t.z]
                };
                if DragFloat3::new(imgui_ui, im_str!("Position"), &mut pos).speed(0.1).build() {
                    let entity = self.entities.get_mut(entity).expect("invalid entity");
                    entity.location.t.x = pos[0];
                    entity.location.t.y = pos[1];
                    entity.location.t.z = pos[2];
                }
            });
    }
}