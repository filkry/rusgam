use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EHeapType {
    Default,
    Upload,
}

impl EHeapType {
    pub fn d3dtype(&self) -> win::D3D12_HEAP_TYPE {
        match self {
            EHeapType::Default => win::D3D12_HEAP_TYPE_DEFAULT,
            EHeapType::Upload => win::D3D12_HEAP_TYPE_UPLOAD,
        }
    }
}

pub struct SHeapProperties {
    raw: win::D3D12_HEAP_PROPERTIES,
}

impl SHeapProperties {
    pub unsafe fn raw(&self) -> &win::D3D12_HEAP_PROPERTIES {
        &self.raw
    }

    pub fn create(type_: EHeapType) -> Self {
        Self {
            raw: win::D3D12_HEAP_PROPERTIES {
                Type: type_.d3dtype(),
                CPUPageProperty: win::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: win::D3D12_MEMORY_POOL_UNKNOWN,
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

impl TEnumFlags32 for EHeapFlags {
    type TRawType = win::D3D12_HEAP_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            EHeapFlags::ENone => win::D3D12_HEAP_FLAG_NONE,
        }
    }
}

pub type SHeapFlags = SEnumFlags32<EHeapFlags>;
