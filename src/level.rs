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
    entity_inits: Vec<EEntityInit>, // $$$FRK(TODO): write what I need to make SVec serde compatible - difficulty is where does the allocator live?
}

pub struct SLevel {
    owned_entities: SVec<SEntityHandle>,
}

impl SInit {
    pub fn new() -> Self {
        Self {
            entity_inits: Vec::new(),
        }
    }

    pub fn new_from_entities(game_context: &SGameContext, entities: &[SEntityHandle]) -> Self {
        let mut entity_inits = Vec::with_capacity(entities.len());
        for entity in entities {
            entity_inits.push(EEntityInit::new_from_entity(game_context, entity.clone()));
        }

        Self {
            entity_inits,
        }
    }
}

impl SLevel {
    pub fn new(allocator: &SAllocatorRef, game_context: &SGameContext, init: &SInit) -> Result<Self, &'static str> {
        let mut owned_entities = SVec::<SEntityHandle>::new(allocator, init.entity_inits.len(), 0)?;
        for e_init in &init.entity_inits {
            let e = e_init.init(game_context)?;
            owned_entities.push(e);
        }

        Ok(Self{
            owned_entities,
        })
    }

    pub fn destroy(&mut self, game_context: &SGameContext) {
        use render;

        game_context.data_bucket.get::<SEntityBVH>()
            .and::<entity_model::SBucket>()
            .and::<entity_animation::SBucket>()
            .and::<render::SRender>()
            .with_mmmm(|bvh, e_model, e_anim, render| {
                render.flush().unwrap();

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