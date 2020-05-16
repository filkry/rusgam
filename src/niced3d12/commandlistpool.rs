use std::rc::Weak;
use std::cell::{RefCell, RefMut};
use std::ops::{DerefMut};

//use safewindows::{break_err};

use super::*;

struct SCommandListPoolList {
    list: RefCell<SCommandList>,
    allocator: SPoolHandle,
}

struct SCommandListPoolActiveAllocator {
    handle: SPoolHandle,
    reusefencevalue: u64,
}

pub struct SCommandListPool {
    queue: Weak<RefCell<SCommandQueue>>,

    allocators: SPool<SCommandAllocator>,
    lists: SPool<SCommandListPoolList>,

    activefence: SFence,
    activeallocators: Vec<SCommandListPoolActiveAllocator>,
}

impl SCommandListPool {
    pub fn create(
        device: &SDevice,
        queue: Weak<RefCell<SCommandQueue>>,
        winapi: &safewindows::SWinAPI,
        num_lists: u16,
        num_allocators: u16,
    ) -> Result<Self, &'static str> {
        assert!(num_allocators > 0 && num_lists > 0);

        let type_ = queue.upgrade().expect("queue dropped before list pool").borrow().type_();

        let mut allocators = Vec::new();
        let mut lists = Vec::new();

        for _ in 0..num_allocators {
            allocators.push(device.create_command_allocator(type_)?);
        }

        for _ in 0..num_lists {
            let mut list = unsafe { device.create_command_list(&mut allocators[0])? };
            // -- immediately close handle because we'll re-assign a new allocator from the pool when ready
            list.close()?;
            lists.push(SCommandListPoolList {
                list: RefCell::new(list),
                allocator: Default::default(),
            });
        }

        Ok(Self {
            queue: queue,
            allocators: SPool::<SCommandAllocator>::create_from_vec(0, num_allocators, allocators),
            lists: SPool::<SCommandListPoolList>::create_from_vec(1, num_lists, lists),
            activefence: device.create_fence(winapi)?,
            activeallocators: Vec::<SCommandListPoolActiveAllocator>::with_capacity(
                num_allocators as usize,
            ),
        })
    }

    pub fn num_free_allocators(&self) -> usize {
        return self.allocators.free_count();
    }

    pub fn free_allocators(&mut self) {
        let completedvalue = self.activefence.completed_value();
        for alloc in &self.activeallocators {
            if alloc.reusefencevalue <= completedvalue {
                self.allocators.free(alloc.handle);
            }
        }

        self.activeallocators
            .retain(|alloc| alloc.reusefencevalue > completedvalue);
    }

    pub fn alloc_list(&mut self) -> Result<SPoolHandle, &'static str> {
        self.free_allocators();

        if self.lists.full() || self.allocators.full() {
            break_err!(Err("no available command list or allocator"));
        }

        let allocatorhandle = self.allocators.alloc()?;
        let allocator = self.allocators.get_mut(allocatorhandle)?;
        allocator.reset();

        let listhandle = self.lists.alloc()?;
        let list = self.lists.get_mut(listhandle)?;
        list.list.borrow_mut().reset(allocator)?;
        list.allocator = allocatorhandle;

        Ok(listhandle)
    }

    pub fn get_list(&self, handle: SPoolHandle) -> Result<RefMut<SCommandList>, &'static str> {
        let list = self.lists.get(handle)?;
        Ok(list.list.borrow_mut())
    }

    pub fn execute_and_free_list(&mut self, handle: SPoolHandle) -> Result<u64, &'static str> {
        let queue = self.queue.upgrade().expect("dropped queue before list");

        let allocator = {
            let list = self.lists.get_mut(handle)?;
            assert!(list.list.borrow().get_type() == queue.borrow().type_());
            queue.borrow().execute_command_list(list.list.borrow_mut().deref_mut())?;

            assert!(list.allocator.valid());
            list.allocator
        };
        self.lists.free(handle);

        let fenceval = queue.borrow().signal(&mut self.activefence)?;

        self.activeallocators.push(SCommandListPoolActiveAllocator {
            handle: allocator,
            reusefencevalue: fenceval,
        });

        Ok(fenceval)
    }

    pub fn wait_for_internal_fence_value(&self, value: u64) {
        self.activefence.wait_for_value(value);
    }

    pub fn flush_blocking(&mut self) -> Result<(), &'static str> {
        let queue = self.queue.upgrade().expect("queue dropped before list pool");
        let result = queue.borrow_mut().flush_blocking();
        result
    }

    pub fn get_internal_fence(&self) -> &SFence {
        &self.activefence
    }

    pub fn gpu_wait(&self, fence: &SFence, value: u64) -> Result<(), &'static str> {
        let queue = self.queue.upgrade().expect("queue dropped before list pool");
        queue.borrow().gpu_wait(fence, value)?;
        Ok(())
    }
}
