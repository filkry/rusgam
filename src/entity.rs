use allocate::{TMemAllocator, SMemVec};
use model::SModel;
use utils::{STransform};
use collections::{SStoragePool, SPoolHandle};

#[allow(dead_code)]
struct SEntity {
    location: STransform,
    model: Option<SModel>,
}

#[allow(dead_code)]
pub struct SEntityBucket {
    entities: SStoragePool<SEntity>,
}

impl SEntity {
    pub fn new() -> Self {
        Self {
            location: STransform::default(),
            model: None,
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

    pub fn set_entity_location(&mut self, entity: SPoolHandle, location: STransform) {
        self.entities.get_mut(entity).expect("invalid entity").location = location;
    }

    pub fn set_entity_model(&mut self, entity: SPoolHandle, model: SModel) {
        self.entities.get_mut(entity).expect("invalid entity").model = Some(model);
    }

    pub fn build_render_data<'a>(&self, allocator: &'a dyn TMemAllocator) -> (SMemVec<'a, STransform>, SMemVec<'a, SModel>) {
        let mut transforms = SMemVec::<STransform>::new(allocator, self.entities.used(), 0).expect("alloc fail");
        let mut models = SMemVec::<SModel>::new(allocator, self.entities.used(), 0).expect("alloc fail");

        for entity_idx in 0..self.entities.max() {
            if let Ok(Some(e)) = self.entities.get_by_index(entity_idx) {
                if let Some(m) = e.model {
                    transforms.push(e.location);
                    models.push(m);
                }
            }
        }

        (transforms, models)
    }
}