use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};
use atom::Atom;

fn bench_write_heavy_small_op(c: &mut Criterion) {
	let b = 1000;

	let mut group = c.benchmark_group("write_heavy_small_op");
	for (i, size) in [b, 2 * b, 4 * b].iter().enumerate() {
		group.throughput(Throughput::Elements(*size as u64));


		group.bench_with_input(BenchmarkId::new("atom", size), &i, |b, _| {
			b.iter(|| {
				black_box({
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
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + mutex", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let arc = std::sync::Arc::new(std::sync::Mutex::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t1_arc.lock().unwrap();
							x[0] += 1;
						}
					});
					let t2 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t2_arc.lock().unwrap();
							x[1] += 2;
						}
					});
					let t3 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t3_arc.lock().unwrap();
							x[0] += 3;
						}
					});
					let t4 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t4_arc.lock().unwrap();
							x[1] += 4;
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + rwlock", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let arc = std::sync::Arc::new(std::sync::RwLock::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t1_arc.write().unwrap();
							x[0] += 1;
						}
					});
					let t2 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t2_arc.write().unwrap();
							x[1] += 2;
						}
					});
					let t3 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t3_arc.write().unwrap();
							x[0] += 3;
						}
					});
					let t4 = std::thread::spawn(move || {
						for _ in 0..1000 {
							let mut x = t4_arc.write().unwrap();
							x[1] += 4;
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});
	}

	group.finish();
}

fn bench_read_heavy_small_op(c: &mut Criterion) {
	let b = 1000;

	let mut group = c.benchmark_group("read_heavy_small_op");
	for (i, size) in [b, 2 * b, 4 * b].iter().enumerate() {
		group.throughput(Throughput::Elements(*size as u64));


		group.bench_with_input(BenchmarkId::new("atom", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let atom = Atom::new(vec![0, 0]);
					let t1_atom = atom.clone();
					let t2_atom = atom.clone();
					let t3_atom = atom.clone();
					let t4_atom = atom.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								t1_atom.mutate(|x| x[0] += 1);
							} else {
								let value = t1_atom.get();
								tvec.push(value);
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								t2_atom.mutate(|x| x[1] += 2);
							} else {
								let value = t2_atom.get();
								tvec.push(value);
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								t3_atom.mutate(|x| x[0] += 3);
							} else {
								let value = t3_atom.get();
								tvec.push(value);
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								t4_atom.mutate(|x| x[1] += 4);
							} else {
								let value = t4_atom.get();
								tvec.push(value);
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + mutex", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let arc = std::sync::Arc::new(std::sync::Mutex::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t1_arc.lock() {
									Ok(mut x) => x[0] += 1,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t1_arc.lock() {
									Ok(value) => tvec.push(value.clone()[0]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t2_arc.lock() {
									Ok(mut x) => x[1] += 2,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t2_arc.lock() {
									Ok(value) => tvec.push(value.clone()[1]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t3_arc.lock() {
									Ok(mut x) => x[0] += 3,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t3_arc.lock() {
									Ok(value) => tvec.push(value.clone()[0]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t4_arc.lock() {
									Ok(mut x) => x[1] += 4,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t4_arc.lock() {
									Ok(value) => tvec.push(value.clone()[1]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + rwlock", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let arc = std::sync::Arc::new(std::sync::RwLock::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t1_arc.write() {
									Ok(mut x) => x[0] += 1,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t1_arc.read() {
									Ok(value) => tvec.push(value.clone()[0]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t2_arc.write() {
									Ok(mut x) => x[1] += 2,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t2_arc.read() {
									Ok(value) => tvec.push(value.clone()[1]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t3_arc.write() {
									Ok(mut x) => x[0] += 3,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t3_arc.read() {
									Ok(value) => tvec.push(value.clone()[0]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec: Vec<i32> = Vec::new();
						for i in 0..1000 {
							if i % 10 == 0 {
								match t4_arc.write() {
									Ok(mut x) => x[1] += 4,
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t4_arc.read() {
									Ok(value) => tvec.push(value.clone()[1]),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});
	}

	group.finish();
}

fn fib(n: u64) -> u64 {
	if n <= 1 {
		return n;
	}
	fib(n - 1) + fib(n - 2)
}

fn bench_read_heavy_big_op(c: &mut Criterion) {
	let b = 1000;

	let mut group = c.benchmark_group("read_heavy_big_op");
	for (i, size) in [b, 2 * b, 4 * b].iter().enumerate() {
		group.throughput(Throughput::Elements(*size as u64));


		group.bench_with_input(BenchmarkId::new("atom", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let atom = Atom::new(vec![0, 0]);
					let t1_atom = atom.clone();
					let t2_atom = atom.clone();
					let t3_atom = atom.clone();
					let t4_atom = atom.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								t1_atom.mutate(|x| {x[0] += fib(20)});
							} else {
								let value = t1_atom.get();
								tvec.push(value);
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								t2_atom.mutate(|x| {x[1] += fib(20)});
							} else {
								let value = t2_atom.get();
								tvec.push(value);
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								t3_atom.mutate(|x| {x[0] += fib(20)});
							} else {
								let value = t3_atom.get();
								tvec.push(value);
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								t4_atom.mutate(|x| {x[1] += fib(20)});
							} else {
								let value = t4_atom.get();
								tvec.push(value);
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				})
			});
		});

		group.bench_with_input(BenchmarkId::new("arc + mutex", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					use std::sync::{Arc, Mutex};
					let arc = Arc::new(Mutex::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t1_arc.lock() {
									Ok(mut x) => {x[0] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t1_arc.lock() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t2_arc.lock() {
									Ok(mut x) => {x[1] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t2_arc.lock() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t3_arc.lock() {
									Ok(mut x) => {x[0] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t3_arc.lock() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t4_arc.lock() {
									Ok(mut x) => {x[1] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t4_arc.lock() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + rwlock", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					use std::sync::{Arc, RwLock};
					let arc = Arc::new(RwLock::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t1_arc.write() {
									Ok(mut x) => {x[0] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t1_arc.read() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t2_arc.write() {
									Ok(mut x) => {x[1] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t2_arc.read() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t3_arc.write() {
									Ok(mut x) => {x[0] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t3_arc.read() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						let mut tvec = Vec::new();
						for i in 0..50 {
							if i % 10 == 0 {
								match t4_arc.write() {
									Ok(mut x) => {x[1] += fib(20)},
									Err(_) => panic!("lock failed"),
								}
							} else {
								match t4_arc.read() {
									Ok(value) => tvec.push(value.clone()),
									Err(_) => panic!("lock failed"),
								}
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});
	}

	group.finish();
}

fn bench_write_heavy_big_op(c: &mut Criterion) {
	let b = 1000;

	let mut group = c.benchmark_group("read_heavy_big_op");
	for (i, size) in [b, 2 * b, 4 * b].iter().enumerate() {
		group.throughput(Throughput::Elements(*size as u64));


		group.bench_with_input(BenchmarkId::new("atom", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					let atom = Atom::new(vec![0, 0]);
					let t1_atom = atom.clone();
					let t2_atom = atom.clone();
					let t3_atom = atom.clone();
					let t4_atom = atom.clone();
					let t1 = std::thread::spawn(move || {
						for _ in 0..50 {
							t1_atom.mutate(|x| {x[0] += fib(20)});
						}
					});
					let t2 = std::thread::spawn(move || {
						for _ in 0..50 {
							t2_atom.mutate(|x| {x[1] += fib(20)});
						}
					});
					let t3 = std::thread::spawn(move || {
						for _ in 0..50 {
							t3_atom.mutate(|x| {x[0] += fib(20)});
						}
					});
					let t4 = std::thread::spawn(move || {
						for _ in 0..50 {
							t4_atom.mutate(|x| {x[1] += fib(20)});
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				})
			});
		});

		group.bench_with_input(BenchmarkId::new("arc + mutex", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					use std::sync::{Arc, Mutex};
					let arc = Arc::new(Mutex::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t1_arc.lock() {
								Ok(mut x) => {x[0] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t2_arc.lock() {
								Ok(mut x) => {x[1] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t3_arc.lock() {
								Ok(mut x) => {x[0] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t4_arc.lock() {
								Ok(mut x) => {x[1] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});

		group.bench_with_input(BenchmarkId::new("arc + rwlock", size), &i, |b, _| {
			b.iter(|| {
				black_box({
					use std::sync::{Arc, RwLock};
					let arc = Arc::new(RwLock::new(vec![0, 0]));
					let t1_arc = arc.clone();
					let t2_arc = arc.clone();
					let t3_arc = arc.clone();
					let t4_arc = arc.clone();
					let t1 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t1_arc.write() {
								Ok(mut x) => {x[0] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t2 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t2_arc.write() {
								Ok(mut x) => {x[1] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t3 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t3_arc.write() {
								Ok(mut x) => {x[0] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					let t4 = std::thread::spawn(move || {
						for _ in 0..50 {
							match t4_arc.write() {
								Ok(mut x) => {x[1] += fib(20)},
								Err(_) => panic!("lock failed"),
							}
						}
					});
					t1.join().unwrap();
					t2.join().unwrap();
					t3.join().unwrap();
					t4.join().unwrap();
				});
			})
		});
	}

	group.finish();
}

criterion_group!(benches,
	bench_read_heavy_small_op,
	bench_write_heavy_small_op,
	bench_read_heavy_big_op,
	bench_write_heavy_big_op,
);
criterion_main!(benches);