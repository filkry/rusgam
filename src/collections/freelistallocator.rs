
mod manager {
    use utils::{align_up};

    pub struct SAllocation {
        start_offset: usize,
        size: usize,
    }

    #[derive(Copy, Clone, Debug)]
    struct SFreeChunk {
        start_offset: usize,
        size: usize,
    }

    pub struct SManager {
        free_chunks: Vec<SFreeChunk>,
    }

    impl SManager {
        pub fn new(size: usize) -> Self {
            let mut result = Self {
                free_chunks: Vec::new(),
            };

            result.free_chunks.push(SFreeChunk {
                start_offset: 0,
                size: size,
            });

            result
        }

        pub fn alloc(&mut self, size: usize, alignment: usize) -> Result<SAllocation, &'static str> {
            let aligned_size = align_up(size, alignment);

            let mut chunk_idx_res = Err("No chunk large enough for allocation");
            for (i, chunk) in (&self.free_chunks).iter().enumerate() {
                let start = align_up(chunk.start_offset, alignment);
                if (start + aligned_size) < (chunk.start_offset + chunk.size) {
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
                }
                else {
                    self.free_chunks.insert(chunk_idx + 1, new_chunk);
                }
            }

            // -- delete chunk if we used all the memory
            if !used_idx {
                self.free_chunks.remove(chunk_idx);
            }

            Ok(allocation)
        }

        pub fn free(&mut self, alloc: SAllocation) {
            let mut chunk_idx = 0;
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
                }
                else {
                    0
                }
            };
            if chunk_idx > 0 && (self.free_chunks[prev].start_offset + self.free_chunks[prev].size) == alloc.start_offset {
                self.free_chunks[prev].size += alloc.size;
                merged = true;
            }

            // -- merge with next in list
            let next = chunk_idx;
            if next < self.free_chunks.len() && (alloc.start_offset + alloc.size) == self.free_chunks[next].start_offset {
                if !merged {
                    self.free_chunks[next].start_offset = alloc.start_offset;
                    self.free_chunks[next].size += alloc.size;
                    merged = true;
                }
                else {
                    // -- merge prev with next
                    self.free_chunks[prev].size += self.free_chunks[next].size;
                    self.free_chunks.remove(next);
                }
            }

            // -- need to insert new entry
            if !merged {
                self.free_chunks.insert(chunk_idx, SFreeChunk {
                    start_offset: alloc.start_offset,
                    size: alloc.size,
                });
            }
        }
    }

    #[test]
    fn test_basic() {
        let mut allocator = SManager::new(100);

        let allocation = allocator.alloc(1, 1).unwrap();
        assert_eq!(allocation.start_offset, 0);
        assert_eq!(allocation.size, 1);

        allocator.free(allocation);
        println!("{:?}", allocator.free_chunks);
        assert_eq!(allocator.free_chunks.len(), 1);
        assert_eq!(allocator.free_chunks[0].start_offset, 0);
        assert_eq!(allocator.free_chunks[0].size, 100);
    }
}
