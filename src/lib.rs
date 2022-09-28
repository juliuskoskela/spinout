mod spin_lock;
mod atom;
pub use spin_lock::SpinLock;
pub use atom::{Atom, Weak};

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering::*};
use std::ptr::NonNull;
use std::cell::UnsafeCell;
use std::hint::spin_loop;

#[test]
fn ut_atom_map() {
	let atom = Atom::new(vec![1, 2, 3]);
	let sum = atom.map(|x| x.iter().sum::<i32>());
	assert_eq!(sum, 6);
}

#[test]
fn ut_atom_map_mut() {
	let atom = Atom::new(vec![1, 2, 3]);
	let three = atom.map_mut(|x| x.pop().unwrap());
	assert_eq!(three, 3);
	assert_eq!(atom.get(), vec![1, 2]);
}

#[test]
fn ut_atom_lock() {
	let atom = Atom::new(vec![1, 2, 3]);
	atom.lock(|x| x.push(4));
	assert_eq!(atom.get(), vec![1, 2, 3, 4]);
}

#[test]
fn ut_atom_set() {
	let atom = Atom::new(vec![1, 2, 3]);
	atom.set(vec![4, 5, 6]);
	assert_eq!(atom.get(), vec![4, 5, 6]);
}

#[test]
fn ut_atom_get() {
	let atom = Atom::new(vec![1, 2, 3]);
	let vec = atom.get();
	assert_eq!(vec, vec![1, 2, 3]);
}

#[test]
fn ut_atom_new() {
	let atom = Atom::new(vec![1, 2, 3]);
	assert_eq!(atom.get(), vec![1, 2, 3]);
}

#[test]
fn ut_write_vec_4theads_loop() {
	for _ in 0..1000 {
		let atom = Atom::new(vec![0, 0]);
		let t1_atom = atom.clone();
		let t2_atom = atom.clone();
		let t3_atom = atom.clone();
		let t4_atom = atom.clone();
		let t1 = std::thread::spawn(move || {
			for _ in 0..1000 {
				t1_atom.lock(|x| x[0] += 1);
				let _ = t1_atom.get();
			}
		});
		let t2 = std::thread::spawn(move || {
			for _ in 0..1000 {
				let _ = t2_atom.get();
				t2_atom.lock(|x| x[1] += 2);
			}
		});
		let t3 = std::thread::spawn(move || {
			for _ in 0..1000 {
				t3_atom.lock(|x| x[0] += 3);
				let _ = t3_atom.get();
			}
		});
		let t4 = std::thread::spawn(move || {
			for _ in 0..1000 {
				let _ = t4_atom.get();
				t4_atom.lock(|x| x[1] += 4);
			}
		});
		t1.join().unwrap();
		t2.join().unwrap();
		t3.join().unwrap();
		t4.join().unwrap();

		assert_eq!(atom.get(), vec![4000, 6000]);
	}
}
