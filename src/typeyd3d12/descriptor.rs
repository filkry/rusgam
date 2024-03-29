use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EDescriptorHeapType {
    ConstantBufferShaderResourceUnorderedAccess,
    Sampler,
    RenderTarget,
    DepthStencil,
}

impl EDescriptorHeapType {
    pub fn d3dtype(&self) -> win::D3D12_DESCRIPTOR_HEAP_TYPE {
        match self {
            EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess => {
                win::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV
            }
            EDescriptorHeapType::Sampler => win::D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            EDescriptorHeapType::RenderTarget => win::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            EDescriptorHeapType::DepthStencil => win::D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        }
    }
}

#[derive(Copy, Clone)]
pub enum EDescriptorHeapFlags {
    None,
    ShaderVisible,
}

impl TEnumFlags for EDescriptorHeapFlags {
    type TRawType = win::D3D12_DESCRIPTOR_HEAP_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            Self::None => win::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            Self::ShaderVisible => win::D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
        }
    }
}

pub type SDescriptorHeapFlags = SEnumFlags<EDescriptorHeapFlags>;

pub struct SDescriptorHeapDesc {
    pub type_: EDescriptorHeapType,
    pub num_descriptors: usize,
    pub flags: SDescriptorHeapFlags,
    //node_mask: u32,
}

impl SDescriptorHeapDesc {
    pub fn d3dtype(&self) -> win::D3D12_DESCRIPTOR_HEAP_DESC {
        win::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: self.type_.d3dtype(),
            NumDescriptors: self.num_descriptors as u32,
            Flags: self.flags.rawtype(),
            NodeMask: 0,
        }
    }
}

#[derive(Clone)]
pub struct SDescriptorHeap {
    pub type_: EDescriptorHeapType,
    pub(super) heap: win::ID3D12DescriptorHeap,
}

impl SDescriptorHeap {
    pub unsafe fn new_from_raw(
        type_: EDescriptorHeapType,
        raw: win::ID3D12DescriptorHeap,
    ) -> Self {
        Self {
            type_: type_,
            heap: raw,
        }
    }

    pub fn get_cpu_descriptor_handle_for_heap_start(&self) -> SCPUDescriptorHandle {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SCPUDescriptorHandle { handle: start }
    }

    pub fn get_gpu_descriptor_handle_for_heap_start(&self) -> SGPUDescriptorHandle {
        let start = unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() };
        SGPUDescriptorHandle { handle: start }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SCPUDescriptorHandle {
    handle: win::D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl SCPUDescriptorHandle {
    pub unsafe fn raw(&self) -> &win::D3D12_CPU_DESCRIPTOR_HANDLE {
        &self.handle
    }

    pub unsafe fn offset(&self, bytes: usize) -> Self {
        SCPUDescriptorHandle {
            handle: win::D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + bytes,
            },
        }
    }

    pub fn d3dtype(&self) -> win::D3D12_CPU_DESCRIPTOR_HANDLE {
        self.handle
    }
}

#[repr(C)]
pub struct SGPUDescriptorHandle {
    handle: win::D3D12_GPU_DESCRIPTOR_HANDLE,
}

impl std::fmt::Debug for SGPUDescriptorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SGPUDescriptorHandle")
         .field("ptr", &self.handle.ptr)
         .finish()
    }
}

impl SGPUDescriptorHandle {
    pub unsafe fn raw(&self) -> &win::D3D12_GPU_DESCRIPTOR_HANDLE {
        &self.handle
    }

    pub unsafe fn offset(&self, bytes: usize) -> Self {
        SGPUDescriptorHandle {
            handle: win::D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + (bytes as u64),
            },
        }
    }

    pub fn d3dtype(&self) -> win::D3D12_GPU_DESCRIPTOR_HANDLE {
        self.handle
    }
}

pub enum EDescriptorRangeType {
    SRV,
    UAV,
    CBV,
    Sampler,
}

impl EDescriptorRangeType {
    pub fn d3dtype(&self) -> win::D3D12_DESCRIPTOR_RANGE_TYPE {
        match self {
            Self::SRV => win::D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            Self::UAV => win::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            Self::CBV => win::D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
            Self::Sampler => win::D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,
        }
    }
}

pub enum EDescriptorRangeOffset {
    EAppend,
    ENumDecriptors { num: u32 },
}

impl EDescriptorRangeOffset {
    pub fn d3dtype(&self) -> u32 {
        match self {
            Self::EAppend => win::D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            Self::ENumDecriptors { num } => *num,
        }
    }
}

pub struct SDescriptorRange {
    pub range_type: EDescriptorRangeType,
    pub num_descriptors: u32,
    pub base_shader_register: u32,
    pub register_space: u32,
    pub offset_in_descriptors_from_table_start: EDescriptorRangeOffset,
}

impl SDescriptorRange {
    pub fn d3dtype(&self) -> win::D3D12_DESCRIPTOR_RANGE {
        win::D3D12_DESCRIPTOR_RANGE {
            RangeType: self.range_type.d3dtype(),
            NumDescriptors: self.num_descriptors,
            BaseShaderRegister: self.base_shader_register,
            RegisterSpace: self.register_space,
            OffsetInDescriptorsFromTableStart: self
                .offset_in_descriptors_from_table_start
                .d3dtype(),
        }
    }
}
