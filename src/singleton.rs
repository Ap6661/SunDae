use std::{cell::UnsafeCell, mem::MaybeUninit, sync::Mutex};

pub struct Singleton<T> {
    status: UnsafeCell<MaybeUninit<T>>,
    lock: Mutex<bool>,
    f: fn() -> T,
}

impl<T> Singleton<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            status: UnsafeCell::new(MaybeUninit::uninit()),
            lock:  Mutex::new(false),
            f
        }
    }

    fn init(&self) {
        let mut lock = self.lock.lock().unwrap();
        if !*lock {
            let res = (self.f)();
            unsafe {
                (*self.status.get()).write(res);
            }    
        }
        *lock = true;
    }

    pub fn mutate(&self, f: impl Fn(&mut T)) {
        self.init();
        let lock = self.lock.lock().unwrap();
        let r = unsafe { (*self.status.get()).assume_init_mut() };
        f(r);
        drop(lock);
    }
    
    pub fn get(&self) -> &T {
        self.init();
        unsafe { (*self.status.get()).assume_init_ref() }
    }
}

unsafe impl<T> Send for Singleton<T> {}
unsafe impl<T> Sync for Singleton<T> {}
