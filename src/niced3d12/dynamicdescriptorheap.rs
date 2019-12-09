use super::*;

struct SDynamicDescriptorHeap {
    heap_type: t12::EDescriptorHeapType,
    descriptor_heap: SDescriptorHeap,
    max_descriptors: usize,
}

impl SDynamicDescriptorHeap {
    pub fn new(device: &SDevice, heap_type: t12::EDescriptorHeapType, max_descriptors: usize) -> Result<Self, &'static str> {
        let desc = t12::SDescriptorHeapDesc {
            type_ = heap_type,
            num_descriptors = max_descriptors,
            flags: t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::ShaderVisible),
        };

        let descriptor_heap = device.create_descriptor_heap(desc)?;

        Ok(Self {
            heap_type: heap_type,
            descriptor_heap: descriptor_heap,
            max_descriptors: max_descriptors,
        })
    }

    pub fn parse_root_signature(&mut self, root_signature: &SRootSignature) {

        for cache in self.descriptor_table_caches {
            cache.in_root_signature = false;
        }

        let mut current_offset = 0;
        for (i, parameter) in root_signature.desc().parameters.iter().enumerate() {
            if parameter.type_ == t12::ERootParameterType::DesriptorTable {
                if let ERootParameterTypeData::DescriptorTable(table) = parameter.type_data {
                    assert!(table.num_descriptor_ranges == 1); // for simplicity for now
                    let range = &table.descriptor_ranges[0];

                    let cache = self.descriptor_table_caches[i];
                    cache.num_descriptors = range.num_descriptors;
                    cache.base_cached_cpu_descriptor = current_offset;
                    cache.in_root_signature = true;

                    current_offset += cache.num_descriptors;
                }
                else {
                    // -- $$$FRK(TODO): pair these two enums in typey
                    assert!(false);
                }
            }
        }
    }

    pub fn stage_descriptors(
        &mut self,
        root_parameter_index: usize,
        offset_into_descriptor_table: usize,
        num_descriptors: usize,
        start_cpu_descriptor: &SCPUDescriptorHandle,
    ) -> Result<(), &'static str> {
        if num_descriptors > self.num_free_descriptors {
            return Err("Can't allocate this many descriptors.")
        }

        if root_parameter_index > self.max_descriptor_tables {
            return Err("More descriptor tables than we're prepared to hanlde.")
        }

        let cache = &self.descriptor_table_caches[root_parameter_index];
        assert!(cache.in_root_signature);

        if (offset_into_descriptor_table + num_descriptors) > cache.num_descriptors {
            return Err("Trying to put descriptors past the end of the table.");
        }

        for i in 0..num_descriptors {
            self.cached_external_cpu_descriptors[cache.base_cached_cpu_descriptor + i] =
                start_cpu_descriptor.add(i, self.descriptor_heap.descriptorsize);
        }

        cache.needs_commit = true;
    }

    fn compute_stale_descriptor_count(&self) -> usize {
        let mut count = 0;
        for cache in self.descriptor_table_caches {
            if cache.in_root_signature && cache.needs_commit {
                count += cache.num_descriptors;
            }
        }
        count
    }

    pub fn commit_stages_descriptors_for_draw(&mut self) {
        if self.compute_stale_descriptor_count() == 0 {
            return;
        }

        command_list.set_descriptor_heap(self.heap_type, self.descriptor_heap);

        for (root_index, cache) in self.descriptor_table_caches.iter().enumerate() {
            if cache.in_root_signature {
                assert!(self.num_free_handles >= cache.num_descriptors);

                //let base_descriptor = self.cached_external_cpu_descriptors[cache.base_cached_cpu_descriptor];
                let first = cache.base_cached_cpu_descriptor;
                let last = cache.base_cached_cpu_descriptor + cache.num_descriptors;
                let slice = &self.cached_external_cpu_descriptors[first..last]
                device.copy_descriptors(slice, self.current_descriptor);

                command_list.set_graphics_root_descriptor_table(root_index, self.current_internal_gpu_descriptor);

                self.current_internal_cpu_descriptor.add(cache.num_descriptors, self.descriptor_size);
                self.current_internal_gpu_descriptor.add(cache.num_descriptors, self.descriptor_size);

                self.num_free_handles -= cache.num_descriptors;
            }
        }
    }

    pub fn reset(&mut self) {
        self.current_internal_cpu_descriptor = self.descriptor_heap.cpu_handle_heap_start();
        self.current_internal_gpu_descriptor = self.descriptor_heap.gpu_handle_heap_start();
        self.num_free_handles = self.max_descriptors;

        for cache in self.descriptor_table_caches {
            cache.in_root_signature = false;
            cache.needs_commit = false;
        }
    }
}