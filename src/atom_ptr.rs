use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::ptr::NonNull;
use std::cell::UnsafeCell;
use std::hint::spin_loop;

struct AtomInner<T> {
    data: UnsafeCell<T>,
    count: AtomicUsize,
    lock: AtomicBool,
}

pub struct Atom<T> {
    inner: NonNull<AtomInner<T>>
}

impl<T> Atom<T> {
	pub fn new(value: T) -> Self {
		let inner = Box::new(AtomInner {
			data: UnsafeCell::new(value),
			count: AtomicUsize::new(1),
			lock: AtomicBool::new(false),
		});
		Atom {
			inner: NonNull::new(Box::into_raw(inner)).unwrap(),
		}
	}

	pub fn get(&self) -> &T {
		unsafe {
			while self.inner.as_ref().lock.load(Ordering::Acquire) {
				spin_loop();
			}
			self.inner.as_ref().data.get().as_ref().unwrap()
		}
	}

	pub unsafe fn get_unchecked(&self) -> &T {
		self.inner.as_ref().data.get().as_ref().unwrap()
	}

	pub fn mutate(&self, f: impl FnOnce(&mut T)) {
		unsafe {
			// Wait for write lock to be released.
			while self.inner.as_ref().lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire).is_err() {
				spin_loop();
			}
			// Mutate
			f(self.inner.as_ref().data.get().as_mut().unwrap());
			// Release write lock.
			self.inner.as_ref().lock.store(false, Ordering::Release);
		}
	}
}

impl<T> Clone for Atom<T> {
	fn clone(&self) -> Self {
		unsafe {
			// Increment count
			self.inner.as_ref().count.fetch_add(1, Ordering::SeqCst);
			Self {
				inner: self.inner,
			}
		}
	}
}

impl<T> Drop for Atom<T> {
	fn drop(&mut self) {
		unsafe {
			// Decrement count
			if self.inner.as_ref().count.fetch_sub(1, Ordering::SeqCst) == 1 {
				// Drop
				drop(Box::from_raw(self.inner.as_ptr()));
			}
		}
	}
}

impl<T> std::ops::Deref for Atom<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.get()
	}
}

unsafe impl<T> Send for Atom<T> {}
unsafe impl<T> Sync for Atom<T> {}

#[test]
fn ut_mutate_vec_4theads() {
	let atom = Atom::new(vec![0, 0]);
	let t1_atom = atom.clone();
	let t2_atom = atom.clone();
	let t3_atom = atom.clone();
	let t4_atom = atom.clone();
	let t1 = std::thread::spawn(move || {
		for _ in 0..1000 {
			t1_atom.mutate(|x| x[0] += 1);
		}
	});
	let t2 = std::thread::spawn(move || {
		for _ in 0..1000 {
			t2_atom.mutate(|x| x[1] += 2);
		}
	});
	let t3 = std::thread::spawn(move || {
		for _ in 0..1000 {
			t3_atom.mutate(|x| x[0] += 3);
		}
	});
	let t4 = std::thread::spawn(move || {
		for _ in 0..1000 {
			t4_atom.mutate(|x| x[1] += 4);
		}
	});
	t1.join().unwrap();
	t2.join().unwrap();
	t3.join().unwrap();
	t4.join().unwrap();

	let sizeof_arc = std::mem::size_of::<std::sync::Arc<std::sync::Mutex<Vec<usize>>>>();
	let sizeof_mutex = std::mem::size_of::<std::sync::Mutex<Vec<usize>>>();
	let sizeof_atom = std::mem::size_of::<Atom<Vec<usize>>>();
	println!("sizeof_arc_mutex: {}", sizeof_arc + sizeof_mutex);
	println!("sizeof_atom: {}", sizeof_atom);
	assert_eq!(*atom, vec![4000, 6000]);
}
