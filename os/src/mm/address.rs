use core::{fmt::Display, ops::Range, ops::Add, ops::Sub};


pub const PA_WIDTH_SV39: usize = 56; // Physical address width for Sv39
pub const VA_WIDTH_SV39: usize = 39; // Virtual address width for Sv39

pub const PAGE_SIZE_WIDTH: usize = 12;
pub const PAGE_SIZE_BYTES: usize = 1 << PAGE_SIZE_WIDTH; // 4KB page size
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 -PAGE_SIZE_WIDTH;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 -PAGE_SIZE_WIDTH;
pub const USER_STACK_SIZE: usize = 0x4000;
pub const KERNEL_STACK_SIZE: usize = 0x40000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE_BYTES + 1; // 错误3：：未对齐页
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE_BYTES;
pub const VPN_MASK: usize = (1 << VA_WIDTH_SV39) - 1;
pub fn kernel_stack_position(process_id: usize) -> Range<VirtAddr> {
    let top = TRAMPOLINE - process_id * (KERNEL_STACK_SIZE + PAGE_SIZE_BYTES); // guard page calculated
    let bottom = top - KERNEL_STACK_SIZE;
    VirtAddr(bottom)..VirtAddr(top)
}
pub trait IntoUsizeRange{
    fn into_usize_range(self) -> Range<usize>;
}
macro_rules! usize_wrapper {
    ($name: ident, $mask: expr) => {
        #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
        pub struct $name(pub usize);
        impl IntoUsizeRange for Range<$name> {
            fn into_usize_range(self) -> Range<usize> {
                self.start.0..self.end.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }
        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value & $mask)
            }
        }

        impl Into<usize> for $name {
            fn into(self) -> usize {
                self.0
            }
        }

        impl Add<usize> for $name {
            type Output = Self;

            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs)
            }
        } 
        impl Add<Self> for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl Sub<usize> for $name {
            type Output = Self;

            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 - rhs)
            }
        }

        impl Sub<Self> for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

    };
}

usize_wrapper!(PhysAddr, (1 << PA_WIDTH_SV39) - 1);
usize_wrapper!(PhysPageNumber, (1 << PPN_WIDTH_SV39) - 1);
usize_wrapper!(VirtAddr, (1 << VA_WIDTH_SV39) - 1);
usize_wrapper!(VirtPageNumber, (1 << VPN_WIDTH_SV39) - 1);
impl PhysAddr {
    pub fn ppn(&self) -> PhysPageNumber {
        PhysPageNumber(self.0 >>PAGE_SIZE_WIDTH)
    }
    pub fn next_ppn(&self) -> PhysPageNumber {
        PhysPageNumber((self.0 + (PAGE_SIZE_BYTES) - 1) >> PAGE_SIZE_WIDTH)
    }
}
impl VirtAddr {
    pub fn vpn(&self) -> VirtPageNumber {
        VirtPageNumber((self.0 & VPN_MASK) >>PAGE_SIZE_WIDTH)
    }
    pub fn next_vpn(&self) -> VirtPageNumber {
        VirtPageNumber(((self.0 + (PAGE_SIZE_BYTES) - 1) & VPN_MASK) >> PAGE_SIZE_WIDTH)
    }
}
impl PhysPageNumber{
    pub fn start_addr(&self) -> PhysAddr {
        PhysAddr(self.0 <<PAGE_SIZE_WIDTH)
    }
    pub fn end_addr(&self) -> PhysAddr {
        PhysAddr(self.0 <<PAGE_SIZE_WIDTH) + PAGE_SIZE_BYTES - 1
    }
}
impl VirtPageNumber {
    pub fn start_addr(&self) -> VirtAddr {
        VirtAddr(self.0 <<PAGE_SIZE_WIDTH)
    }
    pub fn end_addr(&self) -> VirtAddr {
        VirtAddr(self.0 <<PAGE_SIZE_WIDTH) + PAGE_SIZE_BYTES
    }
}
impl PhysPageNumber {
    pub fn get<T>(&self) -> &'static T {
        unsafe {
            let addr = self.start_addr().0 as *const T;
            &*addr
        }
    }
    pub fn get_array<T>(&self) -> &'static [T] {
        unsafe {
            let addr = self.start_addr().0 as *mut T;
            core::slice::from_raw_parts(addr, (PAGE_SIZE_BYTES) / core::mem::size_of::<T>())
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe {
            let addr = self.start_addr().0 as *mut T;
            &mut *addr
        }
    }
    pub fn get_mut_array<T>(&self) -> &'static mut [T] {
        unsafe {
            let addr = self.start_addr().0 as *mut T;
            core::slice::from_raw_parts_mut(addr, (PAGE_SIZE_BYTES) / core::mem::size_of::<T>())
        }
    }
}
impl VirtPageNumber {
    pub fn indexes(&self) -> [usize;3] {
        let mut indexes = [0; 3];
        let mut full_index = self.0;
        for i in (0..3).rev() {
            indexes[i] = full_index & 0x1FF;
            full_index >>= 9;
        }
        indexes
    }
}
