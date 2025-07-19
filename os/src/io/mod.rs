pub(crate) mod dtb;

pub fn init(dtb_ptr: *const u8) {
    dtb::init_dtb(dtb_ptr);
}