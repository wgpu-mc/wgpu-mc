use crate::WmRenderer;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cell::RefCell;
use std::cmp::min;
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ptr::drop_in_place;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroupDescriptor, BindGroupEntry};

const ALIGN: usize = 8;

#[derive(Debug)]
///There are a couple bind group layouts which are roughly the same, such as `ssbo` or `matrix` but have slightly different semantics; this
/// is a convenience struct to deduplicate code
pub struct BindableBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl BindableBuffer {
    pub fn new(wm: &WmRenderer, data: &[u8], usage: wgpu::BufferUsages, layout_name: &str) -> Self {
        let pipelines = wm.pipelines.load();
        let layouts = pipelines.bind_group_layouts.read();
        let layout = layouts.get(layout_name).unwrap();

        assert_ne!(data.len(), 0);

        let buffer = wm
            .wgpu_state
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: data,
                usage,
            });

        let bind_group = wm
            .wgpu_state
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self { buffer, bind_group }
    }
}

type WmArenaObject = (*mut u8, unsafe fn(*mut u8));

/// Untyped arena for render passes
pub struct WmArena<'a> {
    heap: RefCell<*mut u8>,
    capacity: RefCell<usize>,
    total_capacity: RefCell<usize>,
    length: RefCell<usize>,
    objects: RefCell<Vec<WmArenaObject>>,
    heaps: RefCell<Vec<(*mut u8, usize)>>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> WmArena<'a> {
    pub fn new(capacity: usize) -> Self {
        let heap = Self::alloc_heap(capacity);

        Self {
            heap: RefCell::new(heap),
            capacity: RefCell::new(capacity),
            total_capacity: RefCell::new(capacity),
            length: RefCell::new(0),
            objects: RefCell::new(Vec::new()),
            heaps: RefCell::new(vec![(heap, capacity)]),
            phantom: PhantomData,
        }
    }

    fn grow(&self, size: usize) {
        let new_heap = Self::alloc_heap(size);

        *self.length.borrow_mut() = 0;
        *self.capacity.borrow_mut() = size;
        *self.total_capacity.borrow_mut() += size;
        *self.heap.borrow_mut() = new_heap;

        self.heaps.borrow_mut().push((new_heap, size));
    }

    fn alloc_heap(size: usize) -> *mut u8 {
        assert!(size > 0);

        unsafe { alloc_zeroed(Layout::from_size_align(size, ALIGN).unwrap()) }
    }

    pub fn alloc<T>(&self, mut t: T) -> &'a mut T {
        let mut length = { *self.length.borrow() };
        let capacity = { *self.capacity.borrow() };

        let heap_end = unsafe { self.heap.borrow().add(*self.length.borrow()) };

        let t_size = size_of::<T>();
        let t_alignment = align_of::<T>();

        let align_offset = heap_end.align_offset(t_alignment);
        assert_ne!(align_offset, usize::MAX);

        let t_allocate_size = t_size + align_offset;

        if length + t_allocate_size > capacity {
            self.grow(min(t_allocate_size, 4096));

            return self.alloc(t);
        }

        //Pointer to where the data will be allocated
        let t_alloc_ptr = unsafe { heap_end.add(align_offset) as *mut T };

        //Bump
        length += t_allocate_size;
        *self.length.borrow_mut() = length;

        //Copy t into the memory location, and forget t
        unsafe {
            std::ptr::copy(&mut t as *mut T, t_alloc_ptr, 1);
        }
        std::mem::forget(t);

        let drop_fn = unsafe {
            std::mem::transmute::<unsafe fn(*mut T), unsafe fn(*mut u8)>(drop_in_place::<T>)
        };

        self.objects
            .borrow_mut()
            .push((t_alloc_ptr as *mut u8, drop_fn));

        unsafe { &mut *t_alloc_ptr }
    }

    pub fn alloc_immutable<T>(&self, mut t: T) -> &'a T {
        let mut length = { *self.length.borrow() };
        let capacity = { *self.capacity.borrow() };

        let heap_end = unsafe { self.heap.borrow().add(*self.length.borrow()) };

        let t_size = size_of::<T>();
        let t_alignment = align_of::<T>();

        let align_offset = heap_end.align_offset(t_alignment);
        assert_ne!(align_offset, usize::MAX);

        let t_allocate_size = t_size + align_offset;

        if length + t_allocate_size > capacity {
            self.grow(min(t_allocate_size, 4096));

            return self.alloc(t);
        }

        //Pointer to where the data will be allocated
        let t_alloc_ptr = unsafe { heap_end.add(align_offset) as *mut T };

        //Bump
        length += t_allocate_size;
        *self.length.borrow_mut() = length;

        //Copy t into the memory location, and forget t
        unsafe {
            std::ptr::copy(&mut t as *mut T, t_alloc_ptr, 1);
        }
        std::mem::forget(t);

        let drop_fn = unsafe {
            std::mem::transmute::<unsafe fn(*mut T), unsafe fn(*mut u8)>(drop_in_place::<T>)
        };

        self.objects
            .borrow_mut()
            .push((t_alloc_ptr as *mut u8, drop_fn));

        unsafe { &*t_alloc_ptr }
    }
}

impl<'a> Drop for WmArena<'a> {
    fn drop(&mut self) {
        self.objects
            .take()
            .iter()
            .for_each(|(ptr, dealloc)| unsafe {
                dealloc(*ptr);
            });

        self.heaps.take().iter().for_each(|heap| unsafe {
            dealloc(heap.0, Layout::from_size_align(heap.1, ALIGN).unwrap());
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
