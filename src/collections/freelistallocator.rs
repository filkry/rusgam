pub mod manager {
    use utils::align_up;

    #[derive(Debug)]
    pub struct SAllocation {
        start_offset: usize,
        size: usize,
        freed: bool, // for debug purposes, if we drop while not freed we assert
    }

    impl Drop for SAllocation {
        fn drop(&mut self) {
            // $$$FRK(TODO): leak city, just asserting is insufficient, need to go back to RAII-y
            assert!(self.freed);
        }
    }

    impl SAllocation {
        pub fn start_offset(&self) -> usize {
            self.start_offset
        }

        pub fn size(&self) -> usize {
            self.size
        }

        pub fn validate(&self) {
            if self.freed {
                panic!("Use after free!");
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct SFreeChunk {
        start_offset: usize,
        size: usize,
    }

    pub struct SManager {
        free_chunks: Vec<SFreeChunk>,
        size: usize,
        free_space: usize, // total, could be fragmented
    }

    impl SManager {
        pub fn new(size: usize) -> Self {
            let mut result = Self {
                free_chunks: Vec::with_capacity(size / 10),
                size: size,
                free_space: size,
            };

            result.free_chunks.push(SFreeChunk {
                start_offset: 0,
                size: size,
            });

            result
        }

        pub fn alloc(
            &mut self,
            size: usize,
            alignment: usize,
        ) -> Result<SAllocation, &'static str> {
            let aligned_size = align_up(size, alignment);

            let mut chunk_idx_res = Err("No chunk large enough for allocation");
            for (i, chunk) in (&self.free_chunks).iter().enumerate() {
                let start = align_up(chunk.start_offset, alignment);
                if (start + aligned_size) <= (chunk.start_offset + chunk.size) {
                    chunk_idx_res = Ok(i);
                    break;
                }
            }
            let chunk_idx = chunk_idx_res?;

            let old_chunk = self.free_chunks[chunk_idx];

            let mut used_idx = false;

            let allocation = SAllocation {
                start_offset: align_up(old_chunk.start_offset, alignment),
                size: aligned_size,
                freed: false,
            };

            // -- insert new chunk for unused memory before allocation
            if old_chunk.start_offset < allocation.start_offset {
                let new_chunk = SFreeChunk {
                    start_offset: old_chunk.start_offset,
                    size: allocation.start_offset - old_chunk.start_offset,
                };
                self.free_chunks[chunk_idx] = new_chunk;
                used_idx = true;
            }

            // -- insert new chunk for unused memory after allocation
            let end_of_old_chunk = old_chunk.start_offset + old_chunk.size;
            let end_of_allocation = allocation.start_offset + allocation.size;
            if end_of_allocation < end_of_old_chunk {
                let new_chunk = SFreeChunk {
                    start_offset: allocation.start_offset + allocation.size,
                    size: end_of_old_chunk - end_of_allocation,
                };

                if !used_idx {
                    self.free_chunks[chunk_idx] = new_chunk;
                    used_idx = true;
                } else {
                    self.free_chunks.insert(chunk_idx + 1, new_chunk);
                }
            }

            // -- delete chunk if we used all the memory
            if !used_idx {
                self.free_chunks.remove(chunk_idx);
            }

            self.free_space -= allocation.size;

            Ok(allocation)
        }

        pub fn free(&mut self, alloc: &mut SAllocation) {
            let mut chunk_idx = self.free_chunks.len();
            for (i, chunk) in (&self.free_chunks).iter().enumerate() {
                if chunk.start_offset > alloc.start_offset {
                    chunk_idx = i;
                    break;
                }
            }

            let mut merged = false;

            // -- merge with previous in list
            let prev = {
                if chunk_idx > 0 {
                    chunk_idx - 1
                } else {
                    0
                }
            };
            if chunk_idx > 0
                && (self.free_chunks[prev].start_offset + self.free_chunks[prev].size)
                    == alloc.start_offset
            {
                self.free_chunks[prev].size += alloc.size;
                merged = true;
            }

            // -- merge with next in list
            let next = chunk_idx;
            if next < self.free_chunks.len()
                && (alloc.start_offset + alloc.size) == self.free_chunks[next].start_offset
            {
                if !merged {
                    self.free_chunks[next].start_offset = alloc.start_offset;
                    self.free_chunks[next].size += alloc.size;
                    merged = true;
                } else {
                    // -- merge prev with next
                    self.free_chunks[prev].size += self.free_chunks[next].size;
                    self.free_chunks.remove(next);
                }
            }

            // -- need to insert new entry
            if !merged {
                self.free_chunks.insert(
                    chunk_idx,
                    SFreeChunk {
                        start_offset: alloc.start_offset,
                        size: alloc.size,
                    },
                );
            }

            self.free_space += alloc.size;
            alloc.freed = true;
        }
    }

    impl Drop for SManager {
        fn drop(&mut self) {
            // -- entire buffer should be free on drop
            assert_eq!(self.free_chunks.len(), 1);
            assert_eq!(self.free_chunks[0].start_offset, 0);
            assert_eq!(self.free_chunks[0].size, self.size);
        }
    }

    #[test]
    fn test_basic() {
        let mut allocator = SManager::new(100);

        let mut allocation = allocator.alloc(1, 1).unwrap();
        assert_eq!(allocation.start_offset, 0);
        assert_eq!(allocation.size, 1);

        allocator.free(&mut allocation);
        assert_eq!(allocator.free_chunks.len(), 1);
        assert_eq!(allocator.free_chunks[0].start_offset, 0);
        assert_eq!(allocator.free_chunks[0].size, 100);
    }

    #[test]
    fn test_multiple() {
        let mut allocator = SManager::new(10);
        let mut allocation1 = allocator.alloc(3, 1).unwrap();
        println!("allocation1: {:?}", allocation1);
        assert_eq!(allocator.free_chunks[0].start_offset, 3);
        assert_eq!(allocator.free_chunks[0].size, 7);

        let mut allocation2 = allocator.alloc(6, 1).unwrap();
        println!("allocation2: {:?}", allocation2);
        assert_eq!(allocator.free_chunks[0].start_offset, 9);
        assert_eq!(allocator.free_chunks[0].size, 1);

        let mut allocation3 = allocator.alloc(1, 1).unwrap();
        println!("allocation3: {:?}", allocation3);
        assert_eq!(allocator.free_chunks.len(), 0);

        let allocation_fail = allocator.alloc(1, 1);
        assert!(allocation_fail.is_err());

        println!("=== free test ===");
        allocator.free(&mut allocation1);
        println!("free_chunks: {:?}", allocator.free_chunks);
        allocator.free(&mut allocation3);

        assert_eq!(allocator.free_chunks.len(), 2);
        assert!(allocator.free_chunks[0].start_offset < allocator.free_chunks[1].start_offset);

        println!("=== Merge test ===");
        println!("free_chunks: {:?}", allocator.free_chunks);
        allocator.free(&mut allocation2);

        assert_eq!(allocator.free_chunks.len(), 1);
        assert_eq!(allocator.free_chunks[0].start_offset, 0);
        assert_eq!(allocator.free_chunks[0].size, 10);
    }

    #[test]
    fn test_align() {
        let mut allocator = SManager::new(32);

        let mut allocation1 = allocator.alloc(4, 4).unwrap();
        assert_eq!(allocation1.start_offset, 0);
        assert_eq!(allocation1.size, 4);

        let mut allocation2 = allocator.alloc(8, 8).unwrap();
        assert_eq!(allocation2.start_offset, 8);
        assert_eq!(allocation2.size, 8);

        assert_eq!(allocator.free_chunks.len(), 2);

        assert_eq!(allocator.free_chunks[0].start_offset, 4);
        assert_eq!(allocator.free_chunks[0].size, 4);
        assert_eq!(allocator.free_chunks[1].start_offset, 16);
        assert_eq!(allocator.free_chunks[1].size, 16);

        allocator.free(&mut allocation1);

        assert_eq!(allocator.free_chunks.len(), 2);
        assert_eq!(allocator.free_chunks[0].start_offset, 0);
        assert_eq!(allocator.free_chunks[0].size, 8);
        assert_eq!(allocator.free_chunks[1].start_offset, 16);
        assert_eq!(allocator.free_chunks[1].size, 16);

        allocator.free(&mut allocation2);
        assert_eq!(allocator.free_chunks.len(), 1);
    }
}
