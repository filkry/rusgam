use super::*;

#[derive(Copy, Clone)]
pub enum EDescriptorHeapType {
    ConstantBufferShaderResourceUnorderedAccess,
    Sampler,
    RenderTarget,
    DepthStencil,
}

impl EDescriptorHeapType {
    pub fn d3dtype(&self) -> u32 {
        match self {
            EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess => {
                D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV
            }
            EDescriptorHeapType::Sampler => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            EDescriptorHeapType::RenderTarget => D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            EDescriptorHeapType::DepthStencil => D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        }
    }
}

#[derive(Clone)]
pub struct SDescriptorHeap {
    pub type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
}

impl SDescriptorHeap {
    pub unsafe fn new_from_raw(type_: EDescriptorHeapType, raw: ComPtr<ID3D12DescriptorHeap>) -> Self {
        Self {
            type_: type_,
            heap: raw,
        }
    }

    pub fn getcpudescriptorhandleforheapstart(&self) -> SDescriptorHandle {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SDescriptorHandle { handle: start }
    }
}

pub struct SDescriptorHandle {
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl SDescriptorHandle {
    pub unsafe fn raw(&self) -> &D3D12_CPU_DESCRIPTOR_HANDLE {
        &self.handle
    }

    pub unsafe fn offset(&self, bytes: usize) -> SDescriptorHandle {
        SDescriptorHandle {
            handle: D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + bytes,
            },
        }
    }
}
pub enum EDescriptorRangeType {
    SRV,
    UAV,
    CBV,
    Sampler,
}

impl EDescriptorRangeType {
    pub fn d3dtype(&self) -> D3D12_DESCRIPTOR_RANGE_TYPE {
        match self {
            Self::SRV => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            Self::UAV => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            Self::CBV => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
            Self::Sampler => D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,
        }
    }
}

pub enum EDescriptorRangeOffset {
    EAppend,
    ENumDecriptors{ num: u32 },
}

impl EDescriptorRangeOffset {
    pub fn d3dtype(&self) -> u32 {
        match self {
            Self::EAppend => D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            Self::ENumDecriptors{num} => *num,
        }
    }
}

pub struct SDescriptorRange {
    range_type: EDescriptorRangeType,
    num_descriptors: u32,
    base_shader_register: u32,
    register_space: u32,
    offset_in_descriptors_from_table_start: EDescriptorRangeOffset,
}

impl SDescriptorRange {
    pub fn d3dtype(&self) -> D3D12_DESCRIPTOR_RANGE {
        D3D12_DESCRIPTOR_RANGE {
            RangeType: self.range_type.d3dtype(),
            NumDescriptors: self.num_descriptors,
            BaseShaderRegister: self.base_shader_register,
            RegisterSpace: self.register_space,
            OffsetInDescriptorsFromTableStart: self.offset_in_descriptors_from_table_start.d3dtype(),
        }
    }
}


