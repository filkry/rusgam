use allocate::{TMemAllocator, SMemVec};
use model::SModel;
use utils::{STransform};
use collections::{SStoragePool, SPoolHandle};
use databucket::{SDataBucket};
use bvh;

#[allow(dead_code)]
struct SEntity {
    debug_name: Option<&'static str>,
    location: STransform,
    model: Option<SModel>,
    bvh_entry: SPoolHandle,
}

#[allow(dead_code)]
pub struct SEntityBucket {
    entities: SStoragePool<SEntity>,
}

impl SEntity {
    pub fn new() -> Self {
        Self {
            debug_name: None,
            location: STransform::default(),
            model: None,
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

    pub fn create_entity(&mut self) -> Result<SPoolHandle, &'static str> {
        self.entities.insert_val(SEntity::new())
    }

    pub fn set_entity_debug_name(&mut self, entity: SPoolHandle, debug_name: &'static str) {
        self.entities.get_mut(entity).expect("invalid entity").debug_name = Some(debug_name);
    }

    pub fn get_entity_location(&self, entity: SPoolHandle) -> STransform {
        self.entities.get(entity).expect("invalid entity").location
    }

    pub fn set_entity_location(&mut self, entity: SPoolHandle, location: STransform, data_bucket: &SDataBucket) {
        self.entities.get_mut(entity).expect("invalid entity").location = location;

        data_bucket.get_bvh().unwrap().with_mut(|bvh: &mut bvh::STree| {
            bvh.do_mutable_thing();
        });
    }

    pub fn get_entity_model(&self, entity: SPoolHandle) -> Option<SModel> {
        self.entities.get(entity).expect("invalid entity").model
    }

    pub fn set_entity_model(&mut self, entity: SPoolHandle, model: SModel) {
        self.entities.get_mut(entity).expect("invalid entity").model = Some(model);
    }

    pub fn get_entity_bvh_entry(&self, entity: SPoolHandle) -> SPoolHandle {
        self.entities.get(entity).expect("invalid entity").bvh_entry
    }

    pub fn set_entity_bvh_entry(&mut self, entity: SPoolHandle, bvh_entry: SPoolHandle) {
        self.entities.get_mut(entity).expect("invalid entity").bvh_entry = bvh_entry;
    }

    pub fn build_render_data<'a>(&self, allocator: &'a dyn TMemAllocator) -> (SMemVec<'a, SPoolHandle>, SMemVec<'a, STransform>, SMemVec<'a, SModel>) {
        let mut entities = SMemVec::<SPoolHandle>::new(allocator, self.entities.used(), 0).expect("alloc fail");
        let mut transforms = SMemVec::<STransform>::new(allocator, self.entities.used(), 0).expect("alloc fail");
        let mut models = SMemVec::<SModel>::new(allocator, self.entities.used(), 0).expect("alloc fail");

        for entity_idx in 0..self.entities.max() {
            if let Ok(Some(e)) = self.entities.get_by_index(entity_idx) {
                if let Some(m) = e.model {
                    entities.push(self.entities.handle_for_index(entity_idx));
                    transforms.push(e.location);
                    models.push(m);
                }
            }
        }

        (entities, transforms, models)
    }

    pub fn show_imgui_window(&mut self, entity: SPoolHandle, imgui_ui: &imgui::Ui) {
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