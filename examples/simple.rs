use spinout::Atom;

fn main() {

    let mut numbers = vec![];

    for i in 1..43 {
        numbers.push(i);
    }

    let numbers = Atom::new(numbers);
    let t1_numbers = numbers.clone();
    let t2_numbers = numbers.clone();

    let results = Atom::new(vec![]);
    let t1_results = results.clone();
    let t2_results = results.clone();

    let t1 = std::thread::spawn(move || {
        while let Some(x) = t1_numbers.map_mut(|x| x.pop()) {
            if x % 2 == 0 {
                t1_results.lock(|v| v.push(x));
            }
        }
    });

    let t2 = std::thread::spawn(move || {
        while let Some(x) = t2_numbers.map_mut(|x| x.pop()) {
            if x % 2 == 0 {
                t2_results.lock(|v| v.push(x));
            }
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    let mut results = results.get();

    results.sort();

    let expected = [
        2, 4, 6, 8, 10, 12, 14, 16,
        18, 20, 22, 24, 26, 28, 30,
        32, 34, 36, 38, 40, 42
    ];

    assert_eq!(results, expected);
}
