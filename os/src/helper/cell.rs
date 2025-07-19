use core::cell::{RefCell, RefMut};


pub struct SingleThreadSafeCell<T>{
    inner: RefCell<T>
}

unsafe impl<T> Sync for SingleThreadSafeCell<T>{}

impl<T> SingleThreadSafeCell<T>{
    pub fn new(v: T) -> Self{
        SingleThreadSafeCell{ inner: RefCell::new(v) }
    }
    #[allow(mismatched_lifetime_syntaxes)]
    pub fn exclusive_access(&self) -> RefMut<T>{
        self.inner.borrow_mut()
    }
}