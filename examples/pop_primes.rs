use spinout::Atom;

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

fn main() {
	let mut numbers = vec![];
	for i in 0..42 {
		numbers.push(i);
	}
	let numbers = Atom::new(numbers);
	let primes = Atom::new(vec![]);

	let t1_numbers = numbers.clone();
	let t1_primes = primes.clone();
	let t2_numbers = numbers.clone();
	let t2_primes = primes.clone();

	let t1 = std::thread::spawn(move || {
		while let Some(x) = t1_numbers.map_mut(|x| x.pop()) {
			if is_prime(x) {
				t1_primes.lock(|v| v.push(x));
			}
		}
	});
	let t2 = std::thread::spawn(move || {
		while let Some(x) = t2_numbers.map_mut(|x| x.pop()) {
			if is_prime(x) {
				t2_primes.lock(|v| v.push(x));
			}
		}
	});

	t1.join().unwrap();
	t2.join().unwrap();

	let mut primes = primes.get();

	primes.sort();

	let expected = [
		2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41,
	];

	assert_eq!(primes, expected);
}
