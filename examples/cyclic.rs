use spinout::{Atom, Weak};

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

fn main() {
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

	list.push_back(9);
	list.push_back(10);
	list.push_back(11);
	list.push_back(12);
}
