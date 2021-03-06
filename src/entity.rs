use crate::allocate::{SYSTEM_ALLOCATOR};
use crate::collections::{SStoragePool, SPoolHandle};
use crate::entitytypes::{EEntityType};
use crate::math::{Vec3};
use crate::utils::{STransform};
use crate::string_db::{SHashedStr, hash_str};

#[allow(dead_code)]
pub struct SEntity {
    debug_name: Option<SHashedStr>,
    pub type_: EEntityType,
    pub location: STransform,
    pub location_update_frame: u64,
}

#[allow(dead_code)]
pub struct SEntityBucket {
    entities: SStoragePool<SEntity, u16, u16>,
}

pub type SEntityHandle = SPoolHandle<u16, u16>;

impl SEntity {
    pub fn new(type_: EEntityType) -> Self {
        Self {
            debug_name: None,
            type_,
            location: STransform::default(),
            location_update_frame: 0,
        }
    }
}

impl SEntityBucket {
    pub fn new(max_entities: u16) -> Self {
        Self {
            entities: SStoragePool::create(&SYSTEM_ALLOCATOR(), max_entities),
        }
    }

    pub fn create_entity(&mut self, type_: EEntityType) -> Result<SEntityHandle, &'static str> {
        self.entities.insert_val(SEntity::new(type_))
    }

    pub fn purge_entities(&mut self, entities: &[SEntityHandle]) {
        for entity in entities {
            self.entities.free(entity.clone());
        }
    }

    pub fn entity_valid(&self, entity: SEntityHandle) -> bool {
        self.entities.get(entity).is_ok()
    }

    pub fn set_entity_debug_name(&mut self, entity: SEntityHandle, debug_name: &str) {
        self.entities.get_mut(entity).expect("invalid entity").debug_name = Some(hash_str(debug_name));
    }

    pub fn get_entity_debug_name(&self, entity: SEntityHandle) -> &Option<SHashedStr> {
        &self.entities.get(entity).expect("invalid entity").debug_name
    }

    pub fn get_entity_type(&self, entity: SEntityHandle) -> EEntityType {
        self.entities.get(entity).expect("invalid entity").type_
    }

    pub fn get_entity_location(&self, entity: SEntityHandle) -> STransform {
        self.entities.get(entity).expect("invalid entity").location
    }

    pub fn get_location_update_frame(&self, entity: SEntityHandle) -> u64 {
        self.entities.get(entity).expect("invalid entity").location_update_frame
    }

    pub fn set_location(&mut self, gc: &super::SGameContext, entity: SEntityHandle, location: STransform) {
        let entity = self.entities.get_mut(entity).expect("invalid entity");
        entity.location = location;
        entity.location_update_frame = gc.cur_frame;
    }

    pub fn set_position(&mut self, gc: &super::SGameContext, entity: SEntityHandle, position: Vec3) {
        let mut loc = self.get_entity_location(entity);
        loc.t = position;
        self.set_location(gc, entity, loc);
    }

    #[allow(dead_code)]
    pub fn entities(&self) -> &SStoragePool<SEntity, u16, u16> {
        &self.entities
    }

    #[allow(dead_code)]
    pub fn entities_mut(&mut self) -> &mut SStoragePool<SEntity, u16, u16> {
        &mut self.entities
    }

}