use atom::Atom;

fn main() {
	let atom = Atom::new(vec![0, 0]);
	let t1_atom = atom.clone();
	let t2_atom = atom.clone();
	let t3_atom = atom.clone();
	let t4_atom = atom.clone();
	let t1 = std::thread::spawn(move || {
		for _ in 0..10 {
			t1_atom.mutate(|x| x[0] += 1);
		}
	});
	let t2 = std::thread::spawn(move || {
		for _ in 0..10 {
			t2_atom.mutate(|x| x[1] += 2);
		}
	});
	let t3 = std::thread::spawn(move || {
		for _ in 0..10 {
			t3_atom.mutate(|x| x[0] += 3);
		}
	});
	let t4 = std::thread::spawn(move || {
		for _ in 0..10 {
			t4_atom.mutate(|x| x[1] += 4);
		}
	});
	t1.join().unwrap();
	t2.join().unwrap();
	t3.join().unwrap();
	t4.join().unwrap();

	assert_eq!(*atom, vec![40, 60]);
}
