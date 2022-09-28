use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spinout::Atom;
use std::sync::{Arc, Mutex, RwLock};
const UNSORTED_ARR: [i32; 20] = [9, 1, 8, 2, 7, 3, 6, 4, 5, 0, 9, 1, 42, 2, 7, 3, 6, 4, 5, 0];

macro_rules ! make_test_rw {
	($name:ident, $tcnt:expr, $modulo:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});
			}
			group.finish();
		}
	};
}

macro_rules ! make_test_r {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_r($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_r($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_r($tcnt, $multiplier * i);
						})
					})
				});
			}
			group.finish();
		}
	};
}

macro_rules ! make_test_w {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_w($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_w($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_w($tcnt, $multiplier * i);
						})
					})
				});
			}
			group.finish();
		}
	};
}

fn atom_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					tatom.lock(|x| {
						x.sort();
						x.reverse();
					});
				}
				tatom.lock(|x| {
					let y = x.get(0);
					match y {
						Some(fortytwo) => { assert_eq!(fortytwo, &42); },
						None => {},
					}
				});
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn atom_test_w(tcnt: usize, iters: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				tatom.lock(|x| {
					x.sort();
					x.reverse();
				});
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn atom_test_r(tcnt: usize, iters: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				let y = tatom.map(|x| x.iter().find(|x| **x == 42).unwrap().clone());
				assert_eq!(y, 42);
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					match tarc.write() {
						Ok(mut x) => {
							x.sort();
							x.reverse();
						},
						Err(_) => panic!("lock failed"),
					}
				}
				match tarc.read() {
					Ok(x) => {
						let y = x.get(0);
						match y {
							Some(fortytwo) => { assert_eq!(fortytwo, &42); },
							None => {},
						}
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_w(tcnt: usize, iters: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.write() {
					Ok(mut x) => {
						x.sort();
						x.reverse();
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_r(tcnt: usize, iters: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.read() {
					Ok(x) => {
						let y = x.iter().find(|x| **x == 42).unwrap().clone();
						assert_eq!(y, 42);
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					match tarc.lock() {
						Ok(mut x) => {
							x.sort();
							x.reverse();
						},
						Err(_) => panic!("lock failed"),
					}
				}
				match tarc.lock() {
					Ok(x) => {
						let y = x.get(0);
						match y {
							Some(fortytwo) => { assert_eq!(fortytwo, &42); },
							None => {},
						}
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_w(tcnt: usize, iters: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.lock() {
					Ok(mut x) => {
						x.sort();
						x.reverse();
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_r(tcnt: usize, iters: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.lock() {
					Ok(x) => {
						let y = x.iter().find(|x| **x == 42).unwrap().clone();
						assert_eq!(y, 42);
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

// make_test_rw!(t1_small_balanced_rw, 1, 1, 10);
// make_test_rw!(t1_small_read_heavy_rw, 1, 10, 10);
// make_test_r!(t1_small_read_only, 1, 10);
// make_test_w!(t1_small_write_only, 1, 10);

// make_test_rw!(t1_big_balanced_rw, 1, 1, 1000);
// make_test_rw!(t1_big_read_heavy_rw, 1, 10, 1000);
// make_test_r!(t1_big_read_only, 1, 1000);
// make_test_w!(t1_big_write_only, 1, 1000);

make_test_rw!(t4_small_balanced_rw, 4, 1, 10);
make_test_rw!(t4_small_read_heavy_rw, 4, 10, 10);
make_test_r!(t4_small_read_only, 4, 10);
make_test_w!(t4_small_write_only, 4, 10);

make_test_rw!(t4_big_balanced_rw, 4, 1, 1000);
make_test_rw!(t4_big_read_heavy_rw, 4, 10, 1000);
make_test_r!(t4_big_read_only, 4, 1000);
make_test_w!(t4_big_write_only, 4, 1000);

criterion_group!(benches,
	// t1_small_balanced_rw,
	// t1_small_read_heavy_rw,
	// t1_small_read_only,
	// t1_small_write_only,
	// t1_big_balanced_rw,
	// t1_big_read_heavy_rw,
	// t1_big_read_only,
	// t1_big_write_only,
	t4_small_balanced_rw,
	t4_small_read_heavy_rw,
	t4_small_read_only,
	t4_small_write_only,
	t4_big_balanced_rw,
	t4_big_read_heavy_rw,
	t4_big_read_only,
	t4_big_write_only,
);

criterion_main!(benches);