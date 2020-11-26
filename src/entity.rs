use model::{SModelSkinning};
use utils::{STransform};
use collections::{SStoragePool, SPoolHandle};

#[allow(dead_code)]
pub struct SEntity {
    debug_name: Option<&'static str>,
    pub location: STransform,
    pub model_skinning: Option<SModelSkinning>,
    //identity_aabb: Option<SAABB>, // $$$FRK(TODO): ONLY putting this in here right now to avoid moving the renderer!
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
            model_skinning: None,
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

    pub fn set_location(&mut self, entity: SEntityHandle, location: STransform) {
        self.entities.get_mut(entity).expect("invalid entity").location = location;
    }

    #[allow(dead_code)]
    pub fn entities(&self) -> &SStoragePool<SEntity, u16, u16> {
        &self.entities
    }

    #[allow(dead_code)]
    pub fn entities_mut(&mut self) -> &mut SStoragePool<SEntity, u16, u16> {
        &mut self.entities
    }

    pub fn set_entity_model_skinning(&mut self, entity: SEntityHandle, model_skinning: SModelSkinning) {
        let data = self.entities.get_mut(entity).expect("invalid entity");
        data.model_skinning = Some(model_skinning);
    }

    pub fn get_model_skinning(&self, entity: SEntityHandle) -> Option<&SModelSkinning> {
        let data = self.entities.get(entity).expect("invalid entity");
        data.model_skinning.as_ref()
    }

    pub fn get_model_skinning_mut(&mut self, entity: SEntityHandle) -> Option<&mut SModelSkinning> {
        let data = self.entities.get_mut(entity).expect("invalid entity");
        data.model_skinning.as_mut()
    }

    /*
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
    */

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