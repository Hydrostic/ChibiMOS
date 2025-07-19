use buddy_system_allocator::LockedHeap;

const HEAP_SIZE_BITS: usize = 20;
#[global_allocator]
static HEAD_ALLOCATOR: LockedHeap<HEAP_SIZE_BITS> = LockedHeap::empty();

static mut HEAP_SPACE: [u8; 1 << HEAP_SIZE_BITS] = [0; 1 << HEAP_SIZE_BITS];
// 错误1：忘记加 mut
#[allow(static_mut_refs)]
pub fn init_heap() {
    unsafe {
        HEAD_ALLOCATOR.lock().init(HEAP_SPACE.as_ptr() as usize, HEAP_SPACE.len());
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:?}", layout);
}