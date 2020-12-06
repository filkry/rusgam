use serde::{Serialize, Deserialize};

use allocate::{SAllocatorRef};
use collections::{SVec};
use databucket::{SEntityBVH};
use entity::{SEntityHandle};
use entity_animation;
use entity_model;
use entitytypes::{EEntityInit};
use game_context::{SGameContext};

#[derive(Serialize, Deserialize)]
pub struct SInit {
    entities: Vec<EEntityInit>, // $$$FRK(TODO): write what I need to make SVec serde compatible - difficulty is where does the allocator live?
}

pub struct SLevel {
    owned_entities: SVec<SEntityHandle>,
}

impl SInit {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
}

impl SLevel {
    pub fn new(allocator: &SAllocatorRef, game_context: &SGameContext, init: &SInit) -> Result<Self, &'static str> {
        let mut owned_entities = SVec::<SEntityHandle>::new(allocator, init.entities.len(), 0)?;
        for e_init in &init.entities {
            let e = e_init.init(game_context)?;
            owned_entities.push(e);
        }

        Ok(Self{
            owned_entities,
        })
    }

    pub fn destroy(&mut self, game_context: &SGameContext) {
        game_context.data_bucket.get::<SEntityBVH>()
            .and::<entity_model::SBucket>()
            .and::<entity_animation::SBucket>()
            .with_mmm(|bvh, e_model, e_anim| {
                bvh.purge_owners(self.owned_entities.as_ref());
                e_model.purge_entities(self.owned_entities.as_ref());
                e_anim.purge_entities(self.owned_entities.as_ref());
            });

        self.owned_entities.clear();
    }
}

impl Drop for SLevel {
    fn drop(&mut self) {
        assert!(self.owned_entities.len() == 0);
    }
}