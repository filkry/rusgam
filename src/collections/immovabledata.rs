#![allow(dead_code)]

use std::cell::{RefCell, Ref, RefMut};
use std::ffi::c_void;
use std::ops::Deref;

use collections::{SPoolHandle, SStoragePool};

pub struct SImmovableDataRegistryEntry {
    data: *const c_void,
    refcount: u32,
}

pub struct SImmovableDataRegistry {
    datas: RefCell<SStoragePool<SImmovableDataRegistryEntry>>,
}

pub struct SImmovableData<'a, T> {
    pub data: T,
    registry: &'a SImmovableDataRegistry,
    _registered: bool,
    registry_entry: SPoolHandle,
}

pub struct SImmovableDataRef<'a, T> {
    registry: &'a SImmovableDataRegistry,
    registry_entry: SPoolHandle,
    phantom: std::marker::PhantomData<T>,
}

pub struct SImmovableRefCell<'a, T> {
    internal: SImmovableData<'a, RefCell<T>>,
}

pub struct SImmovableRefCellRef<'a, T> {
    internal: SImmovableDataRef<'a, RefCell<T>>,
}

impl SImmovableDataRegistryEntry {
    pub fn new<T>(data: &SImmovableData<T>) -> Self {
        Self{
            data: data as *const SImmovableData<T> as *const c_void,
            refcount: 0,
        }
    }

    pub fn inc(&mut self) {
        self.refcount += 1;
    }

    pub fn dec(&mut self) {
        assert!(self.refcount > 0);
        self.refcount -= 1;
    }
}

impl SImmovableDataRegistry {
    pub fn new(id: u64, max: u16) -> Self {
        Self {
            datas: RefCell::new(SStoragePool::<SImmovableDataRegistryEntry>::create(id, max)),
        }
    }

    pub fn register<T>(&self, data: &SImmovableData<T>) -> Result<SPoolHandle, &'static str> {
        let entry = SImmovableDataRegistryEntry::new(data);
        self.datas.borrow_mut().insert_val(entry)
    }

    pub fn deregister(&self, data_handle: SPoolHandle) {
        assert!(self.datas.borrow().get(data_handle).unwrap().refcount == 0);
        self.datas.borrow_mut().free(data_handle)
    }

    pub fn get_data<'a, T>(&'a self, data_handle: SPoolHandle) -> Result<&SImmovableData<'a, T>, &'static str> {
        let ptr = self.datas.borrow().get(data_handle)?.data;
        unsafe {
            let typed_ptr = ptr as *const SImmovableData<'a, T>;
            return Ok(&*typed_ptr);
        }
    }

    pub fn increase_refcount(&self, data_handle: SPoolHandle) {
        self.datas.borrow_mut().get_mut(data_handle).unwrap().inc();
    }

    pub fn decrease_refcount(&self, data_handle: SPoolHandle) {
        self.datas.borrow_mut().get_mut(data_handle).unwrap().dec();
    }
}

impl<'a, T> SImmovableData<'a, T> {
    pub unsafe fn new(data: T, registry: &'a SImmovableDataRegistry) -> Self {
        Self {
            data,
            registry,
            _registered: false,
            registry_entry: Default::default(),
        }
    }

    // -- once you register, you CANNOT move the type anymore
    pub unsafe fn register(&mut self) -> Result<(), &'static str> {
        self.registry_entry = self.registry.register(&self)?;
        self._registered = true;
        Ok(())
    }

    pub fn get_ref(&self) -> SImmovableDataRef<T> {
        assert!(self._registered == true);
        self.registry.increase_refcount(self.registry_entry);
        SImmovableDataRef {
            registry: self.registry,
            registry_entry: self.registry_entry,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Drop for SImmovableData<'a, T> {
    fn drop(&mut self) {
        self.registry.deregister(self.registry_entry);
    }
}

impl<'a, T> SImmovableDataRef<'a, T> {
    fn get(&self) -> &T {
        self.registry.get_data(self.registry_entry).unwrap().data
    }
}

impl<'a, T> Deref for SImmovableDataRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T> Drop for SImmovableDataRef<'a, T> {
    fn drop(&mut self) {
        self.registry.decrease_refcount(self.registry_entry)
    }
}

impl<'a, T> SImmovableRefCell<'a, T> {
    pub unsafe fn new(data: T, registry: &'a SImmovableDataRegistry) -> Self {
        Self {
            internal: SImmovableData::<'a, RefCell::<T>>::new(
                RefCell::new(data),
                registry,
            ),
        }
    }

    // -- once you register, you CANNOT move the type anymore
    pub unsafe fn register(&mut self) -> Result<(), &'static str> {
        self.internal.register()
    }

    pub fn get_ref(&self) -> SImmovableRefCellRef<T> {
        SImmovableRefCellRef {
            internal: self.internal.get_ref(),
        }
    }
}

impl<'a, T> SImmovableRefCellRef<'a, T> {
    pub fn borrow(&self) -> Ref<T> {
        self.internal.get().borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.internal.get().borrow_mut()
    }
}
