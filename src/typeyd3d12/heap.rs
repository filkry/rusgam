use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EHeapType {
    Default,
    Upload,
}

impl EHeapType {
    pub fn d3dtype(&self) -> D3D12_HEAP_TYPE {
        match self {
            EHeapType::Default => D3D12_HEAP_TYPE_DEFAULT,
            EHeapType::Upload => D3D12_HEAP_TYPE_UPLOAD,
        }
    }
}

pub struct SHeapProperties {
    raw: D3D12_HEAP_PROPERTIES,
}

impl SHeapProperties {
    pub unsafe fn raw(&self) -> &D3D12_HEAP_PROPERTIES {
        &self.raw
    }

    pub fn create(type_: EHeapType) -> Self {
        Self {
            raw: D3D12_HEAP_PROPERTIES {
                Type: type_.d3dtype(),
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EHeapFlags {
    ENone,
}

impl TD3DFlags32 for EHeapFlags {
    type TD3DType = D3D12_HEAP_FLAGS;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            EHeapFlags::ENone => D3D12_HEAP_FLAG_NONE,
        }
    }
}

pub type SHeapFlags = SD3DFlags32<EHeapFlags>;
