use alloc::string::String;
use alloc::vec::Vec;
use dtb_walker::{Dtb, DtbObj, Property, WalkOperation};
use lazy_static::lazy_static;
use core::ops::Range;
use dtb_walker::utils::indent;
use crate::helper::cell::SingleThreadSafeCell;

#[derive(Clone)]
pub struct DeviceTree{
    pub memory: Vec<Range<usize>>
}
pub enum DeviceType {
    Memory,
}
lazy_static!{
    pub static ref DEVICE_TREE: SingleThreadSafeCell<Option<DeviceTree>> = SingleThreadSafeCell::new(None);
}


impl DeviceTree {
    pub fn from_ptr(dtb_ptr: * const u8) -> Self {
        let dtb = unsafe { Dtb::from_raw_parts(dtb_ptr) }.expect("Failed to parse device tree");
        let mut memory_ranges = Vec::new();
        let mut last_device: Option<DeviceType> = None;
        dtb.walk(|path, obj| match obj {
            DtbObj::SubNode { name } => {
                last_device = None;
                // println!("{}{}", indent(path.level(), 2), String::from_utf8_lossy(name));
                WalkOperation::StepInto

            }
            DtbObj::Property(prop) => {
                // println!("{}{:?}", indent(path.level(), 2), prop);
                match prop{ 
                    Property::General { name, value } => {
                        if name.as_bytes() == b"device_type" && &value[0..value.len() - 1] == b"memory" {
                            last_device = Some(DeviceType::Memory);
                        }
                    },
                    Property::Reg(reg) => {
                        if let Some(DeviceType::Memory) = last_device {
                            memory_ranges.extend(reg);
                        }
                    },
                    _ => {}
                }
                WalkOperation::StepOver
            },
        });

        DeviceTree { memory: memory_ranges }
    }
}

pub fn init_dtb(dtb_ptr: *const u8) {
    let mut tree_ref = DEVICE_TREE.exclusive_access();
    if tree_ref.is_some() {
        panic!("[IO] Device tree already initialized");
    }
    tree_ref.replace(DeviceTree::from_ptr(dtb_ptr));
}