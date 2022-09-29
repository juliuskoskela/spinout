use super::*;
use std::thread::Thread;

#[derive(Clone)]
pub struct Park {
	threads: Atom<Vec<Thread>>,
}

impl Park {
	pub fn new() -> Self {
		Self {
			threads: Atom::new(vec![]),
		}
	}

	pub fn park(&self) {
		let thread = std::thread::current();
		self.threads.lock(|threads| threads.push(thread));
		std::thread::park();
	}

	pub fn unpark(&self) {
		self.threads.lock(|threads| {
			match threads.pop() {
				Some(thread) => thread.unpark(),
				None => {},
			}
		});
	}

	pub fn unpark_all(&self) {
		let threads = self.threads.map(|threads| threads.clone());
		for thread in threads {
			thread.unpark();
		}
	}
}
