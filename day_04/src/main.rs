fn main() {
    let mut args = std::env::args().skip(1);

    if args.len() < 2 {
        panic!("Need range! (two args)");
    }

    let min: u64 = args.next().unwrap().parse().unwrap();
    let max: u64 = args.next().unwrap().parse().unwrap();

    println!(
        "Number of possible passwords: {}",
        get_combinations(min, max)
    )
}

enum Detection {
    NoPair,
    PossiblePair,
    AtLeastTriplet,
}

fn check_same_digit(value: &u64) -> bool {
    let mut current = value % 10;
    let mut value = value / 10;
    let mut state = Detection::NoPair;

    while value > 0 {
        let next = value % 10;
        if next == current {
            match state {
                Detection::NoPair => {
                    state = Detection::PossiblePair;
                }
                Detection::PossiblePair => {
                    state = Detection::AtLeastTriplet;
                }
                Detection::AtLeastTriplet => { /* ignore */ }
            }
        } else if let Detection::PossiblePair = state {
            // the third digit is not part of the digit
            return true;
        } else {
            state = Detection::NoPair
        }
        current = next;
        value = value / 10;
    }
    if let Detection::PossiblePair = state {
        true
    } else {
        false
    }
}

/// We check the reverse -> are the digits ever increasing
fn check_left_to_right_decreasing(value: &u64) -> bool {
    let mut current = value % 10;
    let mut value = value / 10;

    while value > 0 {
        let next = value % 10;
        if next > current {
            return false;
        }
        current = next;
        value = value / 10;
    }
    true
}

fn get_combinations(min: u64, max: u64) -> u64 {
    let possible = min..(max + 1);

    possible
        .filter(check_same_digit)
        .filter(check_left_to_right_decreasing)
        .count() as u64
}
