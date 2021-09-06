use super::*;

use arrayvec::ArrayVec;

pub enum EResourceMetadata {
    Invalid,
    SwapChainResource,
    BufferResource { count: usize, sizeofentry: usize },
    Texture2DResource,
}

pub struct SResource {
    pub(super) raw: t12::SResource,

    pub(super) metadata: EResourceMetadata,

    pub(super) debug_name: ArrayVec<[u16; 32]>,
}

pub struct SBufferResource<T> {
    pub raw: SResource,
    pub(super) count: usize,
    pub(super) map_mem: Option<*mut T>,
}

pub struct SBindlessBufferResource<T> {
    pub raw: SBufferResource<T>,
}
pub struct SBindlessBufferResourceSlice {

}


impl SResource {
    pub fn raw(&self) -> &t12::SResource {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut t12::SResource {
        &mut self.raw
    }

    pub fn create_vertex_buffer_view(&self) -> Result<t12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SVertexBufferView::create(
                self.raw.get_gpu_virtual_address(),
                (count * sizeofentry) as u32,
                sizeofentry as u32,
            ))
        } else {
            Err("Trying to create vertexbufferview for non-buffer resource")
        }
    }

    pub fn create_index_buffer_view(
        &self,
        format: t12::EDXGIFormat,
    ) -> Result<t12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SIndexBufferView::create(
                self.raw.get_gpu_virtual_address(),
                format,
                (count * sizeofentry) as u32,
            ))
        } else {
            Err("Trying to create indexbufferview for non-buffer resource")
        }
    }

    pub fn get_required_intermediate_size(
        &self
    ) -> usize {
        unsafe {
            directxgraphicssamples::get_required_intermediate_size(self.raw.raw(), 0, 1) as usize
        }
    }

    pub unsafe fn set_debug_name(&mut self, str_: &'static str) {
        self.raw().raw().SetName(str_).expect("who knows why this would fail");
    }
}

impl<T> SBufferResource<T> {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn map(&mut self) {
        if self.map_mem.is_none() {
            unsafe {
                let cpu_mem = self.raw.raw.map(0, None).unwrap() as *mut T;
                self.map_mem = Some(cpu_mem);
            }
        }
    }

    pub fn copy_to_map(&mut self, data: &[T]) {
        assert!(self.count == data.len());
        assert!(self.map_mem.is_some());

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.map_mem.unwrap(), self.count);
        }
    }

    pub fn create_srv_desc(&self) -> t12::SShaderResourceViewDesc {
        t12::SShaderResourceViewDesc {
            format: t12::EDXGIFormat::Unknown,
            view: t12::ESRV::Buffer(
                t12::SBufferSRV {
                    first_element: 0,
                    num_elements: self.count,
                    structure_byte_stride: std::mem::size_of::<T>(),
                    flags: t12::ED3D12BufferSRVFlags::None,
                },
            ),
        }
    }

    pub fn create_uav_desc(&self) -> t12::SUnorderedAccessViewDesc {
        t12::SUnorderedAccessViewDesc {
            format: t12::EDXGIFormat::Unknown,
            view: t12::EUAV::Buffer(
                t12::SBufferUAV {
                    first_element: 0,
                    num_elements: self.count,
                    structure_byte_stride: std::mem::size_of::<T>(),
                    counter_offset_in_bytes: 0,
                    flags: t12::ED3D12BufferUAVFlags::None,
                },
            ),
        }
    }
}

pub(super) fn update_subresources_stack(
    commandlist: &mut SCommandList,
    destinationresource: &mut SResource,
    intermediateresource: &mut SResource,
    intermediateoffset: u64,
    firstsubresource: u32,
    numsubresources: u32,
    srcdata: &mut t12::SSubResourceData,
) {
    unsafe {
        directxgraphicssamples::UpdateSubresourcesStack(
            commandlist.raw_mut().raw_mut(),
            destinationresource.raw.raw(),
            intermediateresource.raw.raw(),
            intermediateoffset,
            firstsubresource,
            numsubresources,
            srcdata.raw_mut(),
        );
    }
}

impl Default for EResourceMetadata {
    fn default() -> Self {
        EResourceMetadata::Invalid
    }
}

impl t12::SSubResourceData {
    pub fn create_buffer<T>(data: &[T]) -> Self {
        let buffersize = data.len() * std::mem::size_of::<T>();
        unsafe { Self::create(data.as_ptr(), buffersize, buffersize) }
    }

    pub fn create_texture_2d<T>(pixels: &[T], width: usize, height: usize) -> Self {
        assert_eq!(pixels.len(), width * height);
        let row_pitch = width * std::mem::size_of::<T>();
        let slice_pitch = row_pitch * height;
        unsafe { Self::create(pixels.as_ptr(), row_pitch, slice_pitch) }
    }
}
