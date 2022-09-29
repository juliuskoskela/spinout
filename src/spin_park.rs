use super::*;
use std::sync::atomic::AtomicU32;
use crate::futex::{futex_wait, futex_wake};
pub struct SpinPark(AtomicU32);

impl SpinPark {

	/// Create a new `SpinPark`. A spinlock is a simple lock that uses a busy-wait loop
	/// to wait for the lock to be released. This can be advantageous in cases of low
	/// contention i.e. when the lock is only held for a short time and there are few
	/// threads competing for the lock.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinPark;
	///
	/// let lock = SpinPark::new();
	/// ```
	#[inline]
	pub fn new() -> Self {
		SpinPark(AtomicU32::new(0))
	}

	/// Lock the `SpinPark`. This is a blocking operation. If the lock is held by another
	/// thread, this function will spin until the lock is released and then acquire it.
	/// immediately. This function will return once the lock has been acquired.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinPark;
	/// use std::sync::Arc;
	///
	/// let lock = Arc::new(SpinPark::new());
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
            self.lock_contended();
        }
    }

    #[cold]
    fn lock_contended(&self) {
        // Spin first to speed things up if the lock is released quickly.
        let mut state = self.spin();

        // If it's unlocked now, attempt to take the lock
        // without marking it as contended.
        if state == 0 {
            match self.0.compare_exchange(0, 1, Acquire, Relaxed) {
                Ok(_) => return, // Locked!
                Err(s) => state = s,
            }
        }

        loop {
            // Put the lock in contended state.
            // We avoid an unnecessary write if it as already set to 2,
            // to be friendlier for the caches.
            if state != 2 && self.0.swap(2, Acquire) == 0 {
                // We changed it from 0 to 2, so we just successfully locked it.
                return;
            }

            // Wait for the futex to change state, assuming it is still 2.
            futex_wait(&self.0, 2, None);

            // Spin again after waking up.
            state = self.spin();
        }
    }

	#[inline]
	fn spin(&self) -> u32 {
        let mut spin = 100;
        loop {
            // We only use `load` (and not `swap` or `compare_exchange`)
            // while spinning, to be easier on the caches.
            let state = self.0.load(Relaxed);

            // We stop spinning when the mutex is unlocked (0),
            // but also when it's contended (2).
            if state != 1 || spin == 0 {
                return state;
            }

			std::thread::yield_now();
            spin -= 1;
        }
    }

	/// Unlock the `SpinPark`. This function will unlock the lock and allow other threads
	/// to acquire it.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::SpinPark;
	/// use std::sync::Arc;
	///
	/// let lock = Arc::new(SpinPark::new());
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

impl Default for SpinPark {
	fn default() -> Self {
		Self::new()
	}
}
