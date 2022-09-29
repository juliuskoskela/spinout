use super::*;
use std::sync::atomic::AtomicU32;
use crate::futex::{futex_wait, futex_wake};
pub struct SpinLock(AtomicU32);

impl SpinLock {

	/// Create a new `SpinLock`. A spinlock is a simple lock that uses a busy-wait loop
	/// to wait for the lock to be released. This can be advantageous in cases of low
	/// contention i.e. when the lock is only held for a short time and there are few
	/// threads competing for the lock.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinLock;
	///
	/// let lock = SpinLock::new();
	/// ```
	#[inline]
	pub fn new() -> Self {
		SpinLock(AtomicU32::new(0))
	}

	/// Lock the `SpinLock`. This is a blocking operation. If the lock is held by another
	/// thread, this function will spin until the lock is released and then acquire it.
	/// immediately. This function will return once the lock has been acquired.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinLock;
	/// use std::sync::Arc;
	///
	/// let lock = Arc::new(SpinLock::new());
	/// let lock2 = lock.clone();
	///
	/// let t = std::thread::spawn(move || {
	///     lock2.lock();
	///     // do something
	///     std::thread::sleep(std::time::Duration::from_millis(100));
	///     lock2.unlock();
	/// });
	///
	/// lock.lock();
	/// // do something after the spawned thread has acquired finished.
	/// lock.unlock();
	/// t.join().unwrap();
	/// ```
	#[inline]
    pub fn lock(&self) {
        if self.0.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
			std::thread::sleep(std::time::Duration::from_nanos(1));
            let mut state = self.0.load(Relaxed);
			if state == 0 {
				match self.0.compare_exchange(0, 1, Acquire, Relaxed) {
					Ok(_) => return, // Locked!
					Err(s) => state = s,
				}
			}

			while state == 2 || self.0.swap(2, Acquire) != 0 {
				futex_wait(&self.0, 2, None);
				std::thread::sleep(std::time::Duration::from_nanos(1));
				state = self.0.load(Relaxed);
			}
        }
    }

	/// Unlock the `SpinLock`. This function will unlock the lock and allow other threads
	/// to acquire it.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinLock;
	/// use std::sync::Arc;
	///
	/// let lock = Arc::new(SpinLock::new());
	/// let lock2 = lock.clone();
	///
	/// let t = std::thread::spawn(move || {
	///     lock2.lock();
	///     // do something
	///     std::thread::sleep(std::time::Duration::from_millis(100));
	///     lock2.unlock();
	/// });
	///
	/// lock.lock();
	/// // do something after the spawned thread has acquired finished.
	/// lock.unlock();
	/// t.join().unwrap();
	/// ```
	#[inline]
    pub fn unlock(&self) {
        if self.0.swap(0, Release) == 2 {
            // We only wake up one thread. When that thread locks the mutex, it
            // will mark the mutex as contended (2) (see lock_contended above),
            // which makes sure that any other waiting threads will also be
            // woken up eventually.
            self.wake();
        }
    }

    #[cold]
    fn wake(&self) {
        futex_wake(&self.0);
    }
}

impl Default for SpinLock {
	fn default() -> Self {
		Self::new()
	}
}
