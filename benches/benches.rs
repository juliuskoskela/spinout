use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::Rng;
use spinout::Atom;
use std::sync::{Arc, Mutex, RwLock};
const UNSORTED_ARR: [i32; 20] = [9, 1, 8, 2, 7, 3, 6, 4, 5, 0, 9, 1, 42, 2, 7, 3, 6, 4, 5, 0];

// Vec's sort optimizes for already sorted arrays and we don't want that here.
fn merge_sort_recurse(numbers: &mut [i32]) {
	let len = numbers.len();
	if len <= 1 {
		return;
	}

	let mid = len / 2;
	let (left, right) = numbers.split_at_mut(mid);

	merge_sort_recurse(left);
	merge_sort_recurse(right);

	let mut tmp = Vec::with_capacity(len);

	let mut left_idx = 0;
	let mut right_idx = 0;

	while left_idx < left.len() && right_idx < right.len() {
		if left[left_idx] < right[right_idx] {
			tmp.push(left[left_idx]);
			left_idx += 1;
		} else {
			tmp.push(right[right_idx]);
			right_idx += 1;
		}
	}

	while left_idx < left.len() {
		tmp.push(left[left_idx]);
		left_idx += 1;
	}

	while right_idx < right.len() {
		tmp.push(right[right_idx]);
		right_idx += 1;
	}

	numbers.copy_from_slice(&tmp);
}

fn merge_sort(numbers: &mut Vec<i32>) {
	merge_sort_recurse(numbers.as_mut_slice());
}

macro_rules ! make_test_rw {
	($name:ident, $tcnt:expr, $modulo:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1].iter() {
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
			for i in [1].iter() {
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
			for i in [1].iter() {
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

macro_rules ! make_test_rand {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_random_lock($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_random_lock($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_random_lock($tcnt, $multiplier * i);
						})
					})
				});
			}
			group.finish();
		}
	};
}

macro_rules ! make_test_primes {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_sieve_primes($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_sieve_primes($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("NORMAL", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							normal_test_sieve_primes($multiplier * i);
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
                        merge_sort(x);
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

fn atom_test_random_lock(tcnt: usize, iters: usize) {
    let atoms = vec![Atom::new(0); 3];

    let mut threads = Vec::new();
    for _ in 0..tcnt {
        let tatoms = atoms.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..iters {
                for atom in tatoms.iter() {
					atom.lock(|_| {
						let nap_time = rand::thread_rng().gen::<u64>() % 50;
						std::thread::sleep(std::time::Duration::from_nanos(nap_time));
					});
				}
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

fn mutex_test_random_lock(tcnt: usize, iters: usize) {
	let mutexes = vec![Arc::new(Mutex::new(0)); 3];

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tmutexes = mutexes.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				for mutex in tmutexes.iter() {
					let mut lock = mutex.lock().unwrap();
					*lock = rand::thread_rng().gen::<u64>() % 50;
					std::thread::sleep(std::time::Duration::from_nanos(*lock));
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_random_lock(tcnt: usize, iters: usize) {
	let rwlocks = vec![Arc::new(RwLock::new(0)); 3];

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let trwlocks = rwlocks.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				for rwlock in trwlocks.iter() {
					let mut lock = rwlock.write().unwrap();
					*lock = rand::thread_rng().gen::<u64>() % 50;
					std::thread::sleep(std::time::Duration::from_nanos(*lock));
				}
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
					merge_sort(x);
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
							merge_sort(&mut x);
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
						merge_sort(&mut x);
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
							merge_sort(&mut x);
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
						merge_sort(&mut x);
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

fn is_prime(n: u64) -> bool {
	if n < 2 {
		return false;
	}
	if n == 2 {
		return true;
	}
	if n % 2 == 0 {
		return false;
	}
	let mut i = 3;
	while i * i <= n {
		if n % i == 0 {
			return false;
		}
		i += 2;
	}
	true
}

fn atom_test_sieve_primes(tcnt: usize, iters: usize) {
	let mut numbers = vec![];
	for i in 0..iters {
		numbers.push(i as u64);
	}

	let numbers = Atom::new(numbers);
	let primes = Atom::new(vec![]);

	let mut threads = Vec::new();

	for _ in 0..tcnt {
		let tnumbers = numbers.clone();
		let tprimes = primes.clone();
		threads.push(std::thread::spawn(move || {
			while let Some(x) = tnumbers.map_mut(|x| x.pop()) {
				if is_prime(x) {
					tprimes.lock(|v| v.push(x));
				}
			}
		}));
	}
}

fn mutex_test_sieve_primes(tcnt: usize, iters: usize) {
	let mut numbers = vec![];
	for i in 0..iters {
		numbers.push(i as u64);
	}

	let numbers = Arc::new(Mutex::new(numbers));
	let primes = Arc::new(Mutex::new(vec![]));

	let mut threads = Vec::new();

	for _ in 0..tcnt {
		let tnumbers = numbers.clone();
		let tprimes = primes.clone();
		threads.push(std::thread::spawn(move || {
			while let Some(x) = tnumbers.lock().unwrap().pop() {
				if is_prime(x) {
					tprimes.lock().unwrap().push(x);
				}
			}
		}));
	}
}

fn normal_test_sieve_primes(iters: usize) {
	let mut numbers = vec![];
	for i in 0..iters {
		numbers.push(i as u64);
	}

	let mut primes = vec![];

	while let Some(x) = numbers.pop() {
		if is_prime(x) {
			primes.push(x);
		}
	}
}

make_test_rw!(t16_big_balanced_rw, 16, 1, 10_000);
make_test_rw!(t16_big_read_heavy_rw, 16, 10, 10_000);
make_test_r!(t16_big_read_only, 16, 10_000);
make_test_w!(t16_big_write_only, 16, 10_000);
make_test_rand!(t16_big_rand, 32, 100);
make_test_primes!(t8_primes, 8, 10000);

criterion_group!(benches,
	t8_primes,
	t16_big_balanced_rw,
	t16_big_read_heavy_rw,
	t16_big_read_only,
	t16_big_write_only,
	t16_big_rand,
);

criterion_main!(benches);