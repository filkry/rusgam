use model::SModel;
use utils::{STransform};
use collections::SStoragePool;

#[allow(dead_code)]
struct SEntity {
    location: STransform,
    model: Option<SModel>,
}

#[allow(dead_code)]
struct SLevel {
    entities: SStoragePool<SEntity>,
}