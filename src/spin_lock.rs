use super::*;

pub struct SpinLock(AtomicBool);

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
		SpinLock(AtomicBool::new(false))
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
	/// std::thread::spawn(move || {
	/// 	lock2.lock();
	/// 	// do something
	/// 	std::thread::sleep(std::time::Duration::from_millis(100));
	/// 	lock2.unlock();
	/// });
	///
	/// lock.lock();
	/// // do something after the spawned thread has acquired finished.
	/// lock.unlock();
	/// ```
	#[inline]
	pub fn lock(&self) {
		while self.0.compare_exchange(false, true, Acquire, Relaxed).is_err() {
			spin_loop();
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
	/// std::thread::spawn(move || {
	/// 	lock2.lock();
	/// 	// do something
	/// 	std::thread::sleep(std::time::Duration::from_millis(100));
	/// 	lock2.unlock();
	/// });
	///
	/// lock.lock();
	/// // do something after the spawned thread has acquired finished.
	/// lock.unlock();
	/// ```
	#[inline]
	pub fn unlock(&self) {
		self.0.store(false, Relaxed);
	}
}