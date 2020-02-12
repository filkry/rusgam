use model::SModel;
use utils::{STransform};
use collections::SStoragePool;

struct SEntity {
    location: STransform,
    model: Option<SModel>,
}

struct SLevel {
    entities: SStoragePool<SEntity>,
}