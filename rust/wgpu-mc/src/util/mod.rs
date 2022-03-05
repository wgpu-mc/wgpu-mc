
use std::alloc::{Layout, alloc_zeroed, dealloc};

use std::mem::size_of;
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
        let heap = unsafe {
            let layout = Layout::from_size_align(
                capacity,
                ALIGN
            ).unwrap();
            alloc_zeroed(layout)
        };

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

    fn grow(&mut self, increase: usize) {
        let new_heap = unsafe {
            let layout = Layout::from_size_align(
                self.capacity + increase,
                ALIGN
            ).unwrap();
            alloc_zeroed(layout)
        };

        self.heaps.push(
            (self.heap, self.capacity)
        );

        self.length = 0;
        self.capacity = increase;
        self.total_capacity += increase;
        self.heap = new_heap;
    }

    pub fn alloc<T>(&mut self, t: T) -> &'a mut T {
        let size = size_of::<T>();
        let aligned_size =
            size + (
                (ALIGN.wrapping_add(
                    -(size as isize) as usize
                )) % ALIGN
            );
        if self.length + aligned_size > self.capacity {
            self.grow(512);
        }
        //Pointer to where the data will be allocated
        let t_ptr = unsafe { self.heap.add(self.length) };
        //Bump
        self.length += aligned_size;
        //Pointer to reference
        let t_ref = unsafe { (t_ptr as *mut T).as_mut().unwrap() };
        //Move `t` into the heap, and forget the zero-initialized T that was returned
        unsafe {
            std::mem::forget(std::mem::replace(t_ref, t))
        }
        let callback = |ptr: *mut T| {
            unsafe { drop_in_place(ptr); }
        };
        let transmuted_callback = unsafe {
            std::mem::transmute::<fn(*mut T), fn(*mut u8)>(callback)
        };
        self.objects.push((t_ptr, transmuted_callback));
        t_ref
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
                    heap.0,
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