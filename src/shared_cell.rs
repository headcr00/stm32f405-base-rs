use cmim;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    result::Result,
};
use crate::MY_SHARED_PER;

pub struct SharedCell<T>{
    pub data : UnsafeCell<MaybeUninit<T>>,
    pub locked: bool
}
impl<T>  SharedCell<T>{
    pub const fn uninit() -> Self
    {
        Self{
            data : UnsafeCell::new(MaybeUninit::<T>::uninit()),
            locked : false
        }
    }
    pub fn initialize(&self, data : T)
    {
        unsafe {
            // Reference to an uninitialized MaybeUninit
            let mu_ref = &mut *self.data.get();

            // Get a pointer to the data, and use ptr::write to avoid
            // viewing or creating a reference to uninitialized data
            let dat_ptr = mu_ref.as_mut_ptr();
            dat_ptr.write(data);
        }
    }
    pub fn get_value(&self) -> Option<T>
    {
        let old = unsafe {
            // Get a pointer to the initialized data
            let mu_ptr = self.data.get();
            mu_ptr.read_volatile().assume_init()
        };
        Some(old)
    }
    pub fn modify<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()>
    {
        let dat_ref = unsafe {
            let mu_ref = &mut *self.data.get();
            let dat_ptr = mu_ref.as_mut_ptr();
            &mut *dat_ptr
        };

        // Call the user's closure, providing access to the data
        let ret = f(dat_ref);
        self.lol();
        Ok(ret)
    }
    fn lol(&self)
    {
        let x = MY_SHARED_PER.get_value();
    }
}
unsafe impl<T> Sync for SharedCell<T>
    where
        T: Send + Sized
{
}
