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
	for _ in 0..10 {
		let atom = Atom::new(vec![1, 1]);
		let t1_atom = atom.clone();
		let t2_atom = atom.clone();
		let t3_atom = atom.clone();
		let t4_atom = atom.clone();
		let t1 = std::thread::spawn(move || {
			for _ in 0..10 {
				let x = t1_atom.map(|x| x[0]);
				t1_atom.lock(|x| x[0] = x[0] + 1);
				assert!(x > 0);
			}
		});
		let t2 = std::thread::spawn(move || {
			for _ in 0..10 {
				let x = t2_atom.map(|x| x[1]);
				t2_atom.lock(|x| x[1] = x[1] + 1);
				assert!(x > 0);
			}
		});
		let t3 = std::thread::spawn(move || {
			for _ in 0..10 {
				let x = t3_atom.map(|x| x[0]);
				t3_atom.lock(|x| x[0] = x[0] + 1);
				assert!(x > 0);
			}
		});
		let t4 = std::thread::spawn(move || {
			for _ in 0..10 {
				let x = t4_atom.map(|x| x[1]);
				t4_atom.lock(|x| x[1] = x[1] + 1);
				assert!(x > 0);
			}
		});
		t1.join().unwrap();
		t2.join().unwrap();
		t3.join().unwrap();
		t4.join().unwrap();

		assert_eq!(atom.get(), vec![21, 21]);
	}
}

#[test]
fn ut_cyclic() {
	struct DoublyLinkedNode {
		value: i32,
		next: Option<Atom<DoublyLinkedNode>>,
		prev: Option<Weak<DoublyLinkedNode>>,
	}

	unsafe impl Send for DoublyLinkedNode {}
	unsafe impl Sync for DoublyLinkedNode {}

	struct DoublyLinkedList {
		head: Option<Atom<DoublyLinkedNode>>,
		tail: Option<Weak<DoublyLinkedNode>>,
	}

	impl DoublyLinkedList {
		fn new() -> Self {
			Self {
				head: None,
				tail: None,
			}
		}

		fn push_back(&mut self, value: i32) {
			let node = Atom::new(DoublyLinkedNode {
				value,
				next: None,
				prev: None,
			});

			if self.head.is_none() {
				self.head = Some(node.clone());
				self.tail = Some(node.downgrade());
			} else {
				let tail = self.tail.as_ref().unwrap().upgrade().unwrap();
				tail.lock(|tail| {
					tail.next = Some(node.clone());
				});
				node.lock(|node| {
					node.prev = Some(tail.downgrade());
				});
				self.tail = Some(node.downgrade());
			}
		}

		fn pop_back(&mut self) -> Option<i32> {
			if self.head.is_none() {
				return None;
			}

			let tail = self.tail.as_ref().unwrap().upgrade().unwrap();
			let prev = tail.map(|tail| {
				tail.prev.clone()
			});

			if let Some(prev) = prev {
				let prev = prev.upgrade().unwrap();
				prev.lock(|prev| {
					prev.next = None;
				});
				self.tail = Some(prev.downgrade());
			} else {
				self.head = None;
				self.tail = None;
			}

			Some(tail.map(|tail| tail.value))
		}

		fn push_front(&mut self, value: i32) {
			let node = Atom::new(DoublyLinkedNode {
				value,
				next: None,
				prev: None,
			});

			if self.head.is_none() {
				self.head = Some(node.clone());
				self.tail = Some(node.downgrade());
			} else {
				let head = self.head.as_ref().unwrap().clone();
				head.lock(|head| {
					head.prev = Some(node.downgrade());
				});
				node.lock(|node| {
					node.next = Some(head);
				});
				self.head = Some(node);
			}
		}

		fn pop_front(&mut self) -> Option<i32> {
			if self.head.is_none() {
				return None;
			}

			let head = self.head.as_ref().unwrap().clone();
			let next = head.map(|head| {
				head.next.clone()
			});

			if let Some(next) = next {
				next.lock(|next| {
					next.prev = None;
				});
				self.head = Some(next);
			} else {
				self.head = None;
				self.tail = None;
			}

			Some(head.map(|head| head.value))
		}

		fn to_vec(&mut self) -> Vec<i32> {
			let mut vec = Vec::new();
			while let Some(node) = self.pop_front() {
				vec.push(node);
			}
			vec
		}
	}

	let mut list = DoublyLinkedList::new();

	list.push_back(1);
	list.push_back(2);
	list.push_back(3);
	list.push_back(4);

	assert_eq!(list.to_vec(), vec![1, 2, 3, 4]);

	list.push_front(5);
	list.push_front(6);
	list.push_front(7);
	list.push_front(8);

	assert_eq!(list.to_vec(), vec![8, 7, 6, 5]);

	assert_eq!(list.pop_back(), None);
}
