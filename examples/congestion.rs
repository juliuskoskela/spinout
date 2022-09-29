use spinout::Atom;
use rand::Rng;

fn atom_test_random_lock(tcnt: usize, iters: usize) {
    let counts = Atom::new(vec![0; tcnt]);
	let pop_this = Atom::new(vec![0; iters]);

    let mut threads = Vec::new();
    for i in 0..tcnt {
        let tcounts = counts.clone();
		let tpop_this = pop_this.clone();
		let tid = i;
        threads.push(std::thread::spawn(move || {
            while let Some(_) = tpop_this.map_mut(|x| x.pop()) {
				tcounts.lock(|x| {
					x[tid] += 1;
					let nap_time = rand::thread_rng().gen_range(0..10);
					std::thread::sleep(std::time::Duration::from_nanos(nap_time));
				});
			}
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }

	let counts = counts.get();
	for i in 0..tcnt {
		println!("Thread {} was incremented {} times", i, counts[i]);
	}
	assert!(counts.iter().sum::<usize>() == iters);
}

fn main() {
	atom_test_random_lock(4, 100_000);
}