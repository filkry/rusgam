use super::*;

impl t12::EDescriptorHeapType {
    pub const COUNT: usize = 4;

    pub fn index(&self) -> usize {
        match self {
            Self::ConstantBufferShaderResourceUnorderedAccess => 0,
            Self::Sampler => 1,
            Self::RenderTarget => 2,
            Self::DepthStencil => 3,
        }
    }
}

pub struct SDescriptorHeap {
    pub(super) raw: t12::SDescriptorHeap,

    pub(super) numdescriptors: usize,
    pub(super) descriptorsize: usize,
    //cpudescriptorhandleforstart: t12::SDescriptorHandle<'heap, 'device>,
}

impl SDescriptorHeap {
    pub fn type_(&self) -> t12::EDescriptorHeapType {
        self.raw.type_
    }

    pub fn cpu_handle_heap_start(&self) -> t12::SCPUDescriptorHandle {
        self.raw.get_cpu_descriptor_handle_for_heap_start()
    }

    pub fn gpu_handle_heap_start(&self) -> t12::SGPUDescriptorHandle {
        self.raw.get_gpu_descriptor_handle_for_heap_start()
    }

    pub fn cpu_handle(&self, index: usize) -> Result<t12::SCPUDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.cpu_handle_heap_start();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }

    pub fn gpu_handle(&self, index: usize) -> Result<t12::SGPUDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.gpu_handle_heap_start();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

impl t12::SCPUDescriptorHandle {
    pub fn add(&self, count: usize, descriptor_size: usize) -> t12::SCPUDescriptorHandle {
        unsafe { self.offset(count * descriptor_size) }
    }
}

impl t12::SGPUDescriptorHandle {
    pub fn add(&self, count: usize, descriptor_size: usize) -> t12::SGPUDescriptorHandle {
        unsafe { self.offset(count * descriptor_size) }
    }
}
