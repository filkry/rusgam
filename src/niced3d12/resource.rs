use super::*;

use arrayvec::ArrayVec;
use rand;

use crate::collections::freelistallocator;

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

pub struct SStagedUpload {
    upload_start_index: usize,
    default_start_index: usize,
    count: usize,
}

pub struct SBindlessBufferResource<T> {
    upload_resource: SBufferResource<T>,
    default_resource: SBufferResource<T>,
    use_state: t12::EResourceStates,

    allocator: freelistallocator::manager::SManager,

    staged_uploads: Vec<SStagedUpload>,
    next_upload_index: usize,

    rid: u64, // -- random ID to verify calls into here
}
pub struct SBindlessBufferResourceSlice<T> {
    allocation: freelistallocator::manager::SAllocation,
    buffer_rid: u64,

    phantom: std::marker::PhantomData<T>,
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

    pub fn copy_to_map_segment(&mut self, data: &[T], start_offset: usize) {
        assert!((start_offset + data.len()) < self.count);
        assert!(self.map_mem.is_some());

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.map_mem.unwrap().add(start_offset), self.count);
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

impl<T> SBindlessBufferResource<T> {
    pub fn new(
        device: &SDevice,
        flags: t12::SResourceFlags,
        use_state: t12::EResourceStates,
        num_items: usize,
        max_queued_changes: usize,
    ) -> Result<Self, &'static str> {
        let upload_resource = device.create_committed_buffer_resource_for_type::<T>(
            t12::EHeapType::Upload,
            t12::SResourceFlags::from(t12::EResourceFlags::DenyShaderResource),
            t12::EResourceStates::GenericRead,
            max_queued_changes,
        )?;
        upload_resource.map();

        Ok(Self {
            upload_resource,
            default_resource: device.create_committed_buffer_resource_for_type::<T>(
                t12::EHeapType::Default,
                flags,
                t12::EResourceStates::CopyDest,
                num_items,
            )?,
            use_state,

            allocator: freelistallocator::manager::SManager::new(num_items),

            staged_uploads: Vec::new(),
            next_upload_index: 0,

            rid: rand::random(),
        })
    }

    pub fn alloc(&mut self, count: usize) -> Result<SBindlessBufferResourceSlice<T>, &'static str> {
        Ok(SBindlessBufferResourceSlice{
            allocation: self.allocator.alloc(count, 1)?,
            buffer_rid: self.rid,
        })
    }

    pub fn free(&mut self, mut slice: SBindlessBufferResourceSlice<T>) {
        assert!(self.rid == slice.buffer_rid);
        self.allocator.free(&mut slice.allocation)
    }

    pub fn copy_to_upload(&mut self, slice: &SBindlessBufferResourceSlice<T>, data: &[T]) {
        assert!(self.rid == slice.buffer_rid);
        assert!(slice.allocation.size() == data.len());
        self.upload_resource.copy_to_map_segment(data, self.next_upload_index);
        self.staged_uploads.push(SStagedUpload {
            upload_start_index: self.next_upload_index,
            default_start_index: slice.allocation.start_offset(),
            count: data.len(),
        });

        self.next_upload_index += data.len();
    }

    fn transition_resources_for_copy(&mut self, list: &mut SCommandList) {
        list.transition_resource(
            &self.upload_resource.raw,
            t12::EResourceStates::GenericRead,
            t12::EResourceStates::CopySource,
        );
        list.transition_resource(
            &self.default_resource.raw,
            self.use_state,
            t12::EResourceStates::CopyDest,
        );
    }

    fn transition_resources_for_use(&mut self, list: &mut SCommandList) {
        list.transition_resource(
            &self.upload_resource.raw,
            t12::EResourceStates::CopySource,
            t12::EResourceStates::GenericRead,
        );
        list.transition_resource(
            &self.default_resource.raw,
            t12::EResourceStates::CopyDest,
            self.use_state,
        );
    }

    pub fn flush_upload_to_default(&mut self, list: &mut SCommandList) {
        self.transition_resources_for_copy(list);

        for staged_upload in self.staged_uploads {
            list.raw_mut().copy_buffer_region_typed(
                &self.default_resource.raw,
                staged_upload.default_start_index,
                &self.upload_resource.raw,
                staged_upload.upload_start_index,
                staged_upload.count,
            );
        }

        self.transition_resources_for_use(list);

        self.staged_uploads.clear();

        self.next_upload_index = 0;
    }

    // -- this is unsafe because it can't be called again until GPU copy work finishes
    // -- $$$FRK(TODO): make it sane to do sync work here without having access a command list pool etc
    pub unsafe fn sync_copy_to_default(
        &mut self,
        list: &mut SCommandList,
        slice: &SBindlessBufferResourceSlice<T>,
        data: &[T])
    {
        assert!(self.rid == slice.buffer_rid);
        assert!(slice.allocation.size() == data.len());
        assert!(self.next_upload_index == 0);

        self.upload_resource.copy_to_map_segment(data, 0);

        self.transition_resources_for_copy(list);

        list.raw_mut().copy_buffer_region_typed(
            &self.default_resource.raw,
            slice.allocation.start_offset(),
            &self.upload_resource.raw,
            0,
            data.len(),
        );

        self.transition_resources_for_use(list);
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
