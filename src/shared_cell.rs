use core::{
    cell::UnsafeCell,
    cmp::PartialEq,
    mem::MaybeUninit,
    result::Result,
    sync::atomic::{AtomicU8, Ordering},
};

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
            // Create a mutable reference to an initialized MaybeUninit
            let mu_ref = &mut *self.data.get();

            // Create a mutable reference to the initialized data behind
            // the MaybeUninit. This is fine, because the scope of this
            // reference can only live to the end of this function, and
            // cannot be captured by the closure used below.
            //
            // Additionally we have a re-entrancy check above, to prevent
            // creating a duplicate &mut to the inner data
            let dat_ptr = mu_ref.as_mut_ptr();
            &mut *dat_ptr
        };

        // Call the user's closure, providing access to the data
        let ret = f(dat_ref);
        Ok(ret)
    }
}
unsafe impl<T> Sync for SharedCell<T>
    where
        T: Send + Sized
{
}
