use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cmp::min;
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ptr::drop_in_place;

const ALIGN: usize = 8;

type WmArenaObject = (*mut u8, unsafe fn(*mut u8));
///Untyped arena for render passes
pub struct WmArena<'a> {
    heap: *mut u8,
    capacity: usize,
    total_capacity: usize,
    length: usize,
    objects: Vec<WmArenaObject>,
    heaps: Vec<(*mut u8, usize)>,
    phantom: PhantomData<&'a ()>,
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
            phantom: PhantomData::default(),
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

        unsafe { alloc_zeroed(Layout::from_size_align(size, ALIGN).unwrap()) }
    }

    pub fn alloc<T>(&mut self, mut t: T) -> &'a mut T {
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
        let t_alloc_ptr = unsafe { heap_end.add(align_offset) as *mut T };

        //Bump
        self.length += t_allocate_size;

        //Copy t into the memory location, and forget t
        unsafe {
            std::ptr::copy(&mut t as *mut T, t_alloc_ptr, 1);
        }
        std::mem::forget(t);

        let drop_fn = unsafe {
            std::mem::transmute::<unsafe fn(*mut T), unsafe fn(*mut u8)>(drop_in_place::<T>)
        };

        self.objects.push((t_alloc_ptr as *mut u8, drop_fn));

        unsafe { &mut *t_alloc_ptr }
    }
}

impl<'a> Drop for WmArena<'a> {
    fn drop(&mut self) {
        self.objects.iter().for_each(|(ptr, dealloc)| unsafe {
            dealloc(*ptr);
        });

        self.heaps.iter().for_each(|heap| unsafe {
            dealloc(
                heap.0 as *mut u8,
                Layout::from_size_align(heap.1, ALIGN).unwrap(),
            );
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
