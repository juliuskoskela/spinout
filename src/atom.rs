use super::*;

struct AtomInner<T: ?Sized> {
	count: (AtomicUsize, AtomicUsize),
    lock: SpinLock,
    data: UnsafeCell<T>,
}

/// A thread-safe reference-counted mutabel pointer.
///
/// # Examples
///
/// ```
/// use spinout::Atom;
///
/// let atom = Atom::new(vec![1, 2, 3]);
/// let sum: i32 = atom.map(|v| v.iter().sum());
/// assert_eq!(sum, 6);
/// ```
pub struct Atom<T: Send + ?Sized> {
    inner: NonNull<AtomInner<T>>,
	phantom: PhantomData<AtomInner<T>>,
}

impl<T: Send> Atom<T> {

	/// Create a new `Atom<T>` with the given value. The type `T` must be `Send`.
	/// `Atom<T>` is a thread-safe reference-counted pointer with interior mutability.
	/// Unlike `Arc<Mutex<T>>`, `Atom<T>` does not use system futexes, but instead uses
	/// a simple spin-lock. This can be advantageous in cases of low contention i.e.
	/// when the lock is only held for a short time and there are few threads
	/// competing for the lock.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(5);
	/// ```
	#[inline]
	pub fn new(value: T) -> Self {
		let inner = Box::new(AtomInner {
			data: UnsafeCell::new(value),
			count: (AtomicUsize::new(1), AtomicUsize::new(1)),
			lock: SpinLock::new(),
		});
		Atom {
			inner: NonNull::new(Box::into_raw(inner)).unwrap(),
			phantom: PhantomData,
		}
	}

	/// Get a copy of the value inside the `Atom<T>`. This is a blocking operation. If the
	/// lock is held by another thread, this function will spin until the lock is released.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(5);
	/// let value = atom.get();
	/// assert_eq!(value, 5);
	/// ```
	#[inline]
	pub fn get(&self) -> T where T: Clone {
		self.map(|x| x.clone())
	}

	/// Set the value inside the `Atom<T>`. This is a blocking operation. If the
	/// lock is held by another thread, this function will spin until the lock is released.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(5);
	/// atom.set(10);
	/// assert_eq!(atom.get(), 10);
	/// ```
	#[inline]
	pub fn set(&self, value: T) {
		self.map_mut(|x| *x = value);
	}

	/// Lock the `Atom<T>` and apply the given function to the value inside. This is a blocking
	/// operation. If the lock is held by another thread, this function will spin until the lock
	/// is released.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(5);
	/// atom.lock(|x| *x += 5);
	/// assert_eq!(atom.get(), 10);
	/// ```
	#[inline]
	pub fn lock(&self, f: impl FnOnce(&mut T)) {
		let inner = unsafe { self.inner.as_ref() };
		inner.lock.lock();
		f(unsafe { inner.data.get().as_mut().unwrap() });
		inner.lock.unlock();
	}

	/// Map a function over the value inside the `Atom<T>` and return the result.
	/// This is a blocking operation. If the lock is held by another thread, this
	/// function will spin until the lock is released.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(vec![1, 2, 3]);
	/// let sum: i32 = atom.map(|x| x.iter().sum());
	/// assert_eq!(sum, 6);
	/// ```
	#[inline]
	pub fn map<U>(&self, f: impl FnOnce(&T) -> U) -> U {
		let inner = unsafe { self.inner.as_ref() };
		inner.lock.lock();
		let data = f(unsafe { inner.data.get().as_mut().unwrap() });
		inner.lock.unlock();
		data
	}

	/// Map a function over the value inside the `Atom<T>` and return the result.
	/// This function allows the value to be mutated. This is a blocking operation.
	/// If the lock is held by another thread, this function will spin until the lock
	/// is released.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(vec![1, 2, 3]);
	/// let three = atom.map_mut(|x| x.pop());
	/// assert_eq!(three, Some(3));
	/// assert_eq!(atom.get(), vec![1, 2]);
	/// ```
	#[inline]
	pub fn map_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
		let inner = unsafe { self.inner.as_ref() };
		inner.lock.lock();
		let data = f(unsafe { inner.data.get().as_mut().unwrap() });
		inner.lock.unlock();
		data
	}

	/// Downgrade the `Atom<T>` to a `Weak<T>`. This is a non-blocking operation.
	/// The `Weak<T>` can be upgraded to an `Atom<T>` using the `upgrade` method.
	/// If the `Atom<T>` is dropped, the `Weak<T>` will no longer be able to be upgraded and
	/// will return `None`.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::Atom;
	///
	/// let atom = Atom::new(5);
	/// let weak = atom.downgrade();
	/// assert_eq!(atom.get(), weak.upgrade().unwrap().get());
	/// ```
	#[inline]
	pub fn downgrade(&self) -> Weak<T> {
		let inner = unsafe { self.inner.as_ref() };
		inner.count.1.fetch_add(1, Relaxed);
		Weak {
			inner: self.inner
		}
	}
}

impl<T: Send> Clone for Atom<T> {
	fn clone(&self) -> Self {
		let inner = unsafe { self.inner.as_ref() };
		if inner.count.0.fetch_add(1, SeqCst) == usize::MAX {
			panic!("Atom count overflow");
		}
		Atom {
			inner: self.inner,
			phantom: PhantomData,
		}
	}
}

impl<T: Send + ?Sized> Drop for Atom<T> {
	fn drop(&mut self) {
		let inner = unsafe { self.inner.as_ref() };
		if inner.count.0.fetch_sub(1, SeqCst) == 1 {
			unsafe { Box::from_raw(self.inner.as_ptr()) };
		}
	}
}

unsafe impl<T: Send> Send for Atom<T> {}
unsafe impl<T: Send> Sync for Atom<T> {}

/// A weak reference to an `Atom`. Weak references do not count towards the
/// strong reference count, and will not prevent the value from being dropped.
/// However, they may be upgraded to strong references. If the value has already
/// been dropped, then an `Option::None` will be returned when attempting to
/// upgrade.
///
/// # Examples
///
/// ```
/// use spinout::{Atom, Weak};
///
/// let atom = Atom::new(3);
/// let weak = atom.downgrade();
/// let three = weak.upgrade().unwrap().get();
/// assert_eq!(three, 3);
/// ```
pub struct Weak<T: Send + ?Sized> {
	inner: NonNull<AtomInner<T>>
}

impl<T: Send + ?Sized> Weak<T> {
	/// Attempt to upgrade the `Weak` reference to a strong `Atom`. If the value
	/// has already been dropped, then an `Option::None` will be returned.
	/// Otherwise, an `Option::Some` will be returned containing the strong
	/// reference.
	///
	/// # Examples
	///
	/// ```
	/// use spinout::{Atom, Weak};
	///
	/// let atom = Atom::new(3);
	/// let weak = atom.downgrade();
	/// let three = weak.upgrade().unwrap().get();
	/// assert_eq!(three, 3);
	/// ```
	pub fn upgrade(&self) -> Option<Atom<T>> {
		let self_ref = unsafe { self.inner.as_ref() };
		let count = self_ref.count.0.load(Acquire);
		if count == 0 {
			None
		} else {
			self_ref.count.0.fetch_add(1, AcqRel);
			Some(Atom {
				inner: self.inner,
				phantom: PhantomData,
			})
		}
	}
}

impl<T: ?Sized + Send> Drop for Weak<T> {
    fn drop(&mut self) {
        // If we find out that we were the last weak pointer, then its time to
        // deallocate the data entirely.
        //
        // It's not necessary to check for the locked state here, because the
        // weak count can only be locked if there was precisely one weak ref,
        // meaning that drop could only subsequently run ON that remaining weak
        // ref, which can only happen after the lock is released.
		let inner = unsafe { self.inner.as_ref() };
        if inner.count.1.fetch_sub(1, Release) == 1 {
			unsafe {
				drop(Box::from_raw(self.inner.as_ptr()));
			}
        }
    }
}

impl<T: ?Sized + Send> Clone for Weak<T> {
	fn clone(&self) -> Weak<T> {
		let inner = unsafe { self.inner.as_ref() };
		inner.count.1.fetch_add(1, Relaxed);
		Weak {
			inner: self.inner
		}
	}
}
