use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll};
use portable_atomic::AtomicBool;

pub struct Mutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>
}

unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.mutex.data.get().as_ref().unwrap_unchecked()
        }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.mutex.data.get().as_mut().unwrap_unchecked()
        }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.swap(false, Ordering::Relaxed);
    }
}

pub struct FutureMutexGuard<'a, T> {
    mutex: &'a Mutex<T>
}

impl<'a, T> Future for FutureMutexGuard<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.locked.swap(true, Ordering::Relaxed) {
            Poll::Pending
        } else {
            Poll::Ready(MutexGuard{ mutex: self.mutex })
        }
    }
}

impl<T> Mutex<T> {

    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data)
        }
    }
    pub fn lock(&self) -> FutureMutexGuard<T> {
        FutureMutexGuard { mutex: self }
    }

}