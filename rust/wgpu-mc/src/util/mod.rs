
use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::cmp::min;

use std::mem::{align_of, ManuallyDrop, size_of};
use std::ptr::drop_in_place;
use std::marker::PhantomData;

const ALIGN: usize = 8;

///Untyped arena for render passes
pub struct WmArena<'a> {
    heap: *mut u8,
    capacity: usize,
    total_capacity: usize,
    length: usize,
    objects: Vec<(*mut u8, fn (*mut u8))>,
    heaps: Vec<(*mut u8, usize)>,
    phantom: PhantomData<&'a ()>
}

impl<'a> WmArena<'a> {

    pub fn new(capacity: usize) -> Self {
        let heap = Self::alloc_heap(capacity);

        Self {
            heap,
            capacity,
            total_capacity: capacity,
            length: 0,
            objects: Vec::new(),
            heaps: vec![(heap, capacity)],
            phantom: PhantomData::default()
        }
    }

    fn grow(&mut self, size: usize) {
        let new_heap = Self::alloc_heap(size);

        self.length = 0;
        self.capacity = size;
        self.total_capacity += size;
        self.heap = new_heap;

        self.heaps.push((new_heap, size));
    }

    fn alloc_heap(size: usize) -> *mut u8 {
        assert!(size > 0);

        unsafe {
            alloc_zeroed(
                Layout::from_size_align(
                    size,
                    ALIGN
                ).unwrap()
            )
        }
    }

    pub fn alloc<T>(&mut self, t: T) -> &'a mut T {
        let heap_end = unsafe { self.heap.add(self.length) };

        let t_size = size_of::<T>();
        let t_alignment = align_of::<T>();

        let align_offset = heap_end.align_offset(t_alignment);
        assert_ne!(align_offset, usize::MAX);

        let t_allocate_size = t_size + align_offset;

        if self.length + t_allocate_size > self.capacity {
            self.grow(min(t_allocate_size, 4096));

            return self.alloc(t);
        }

        //Pointer to where the data will be allocated
        let t_alloc_ptr = unsafe { heap_end.add(align_offset) };

        //Bump
        self.length += t_allocate_size;

        ///SAFETY: This new pointer is up until now unused
        let t_mut_ref = unsafe { (t_alloc_ptr as *mut T).as_mut().unwrap() };

        //Move `t` into the allocated spot and forget the zero-initialized T that was returned
        let uninitialized_t = std::mem::replace(t_mut_ref, t);

        std::mem::forget(uninitialized_t);

        let callback = |ptr: *mut T| {
            ///SAFETY: this will only be called once WmArena is dropped, meaning that there are no
            /// references to this data.
            unsafe { drop_in_place(ptr); }
        };

        let transmuted_callback = unsafe {
            std::mem::transmute::<fn(*mut T), fn(*mut u8)>(callback)
        };

        self.objects.push((t_alloc_ptr, transmuted_callback));

        t_mut_ref
    }

}

impl<'a> Drop for WmArena<'a> {

    fn drop(&mut self) {
        self.objects.iter().for_each(|(ptr, dealloc)| {
            dealloc(*ptr);
        });

        self.heaps.iter().for_each(|heap| {
            unsafe {
                dealloc(
                    heap.0 as *mut u8,
                    Layout::from_size_align(
                        heap.1, ALIGN
                    ).unwrap()
                );
            }
        });
    }

}

// pub struct AVec<T: Send + Sync> {
//     capacity: AtomicUsize,
//     length: AtomicUsize,
//     data: *mut T
// }
//
// impl<T: Send + Sync> AVec<T> {
//     pub fn new(capacity: usize) -> Self {
//         let layout = Layout::array::<T>(capacity)
//             .unwrap();
//         let data = unsafe {
//             std::alloc::alloc(layout) as *mut T
//         };
//
//         Self {
//             capacity: AtomicUsize::new(capacity),
//             length: AtomicUsize::new(0),
//             data
//         }
//     }
//
//     pub fn insert(&self, t: T) {
//         let index = self.length.fetch_add(1, Ordering::Acquire);
//         if index - 1 > self.capacity.fetch_add(0, Ordering::AcqRel) {
//
//         }
//     }
// }