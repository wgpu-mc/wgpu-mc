use std::sync::atomic::AtomicUsize;
use std::alloc::Layout;
use bytemuck::__core::sync::atomic::Ordering;

pub struct AVec<T: Send + Sync> {
    capacity: AtomicUsize,
    length: AtomicUsize,
    data: *mut T
}

impl<T: Send + Sync> AVec<T> {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity)
            .unwrap();
        let data = unsafe {
            std::alloc::alloc(layout) as *mut T
        };

        Self {
            capacity: AtomicUsize::new(capacity),
            length: AtomicUsize::new(0),
            data
        }
    }

    pub fn insert(&self, t: T) {
        let index = self.length.fetch_add(1, Ordering::Acquire);
        if index - 1 > self.capacity.fetch_add(0, Ordering::AcqRel) {

        }
    }
}