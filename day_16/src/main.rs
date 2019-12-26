use std::collections::{HashSet, VecDeque};
use std::fs::read_to_string;

fn get_pattern(needed_len: usize, pos: usize) -> Vec<i64> {
    let mut output = Vec::with_capacity(needed_len);
    let elems = [0, 1, 0, -1];
    let mut idx = 0;

    // beginning
    for _ in 0..pos {
        output.push(elems[idx]);
    }
    idx = 1;

    while output.len() < needed_len {
        for _ in 0..(pos + 1) {
            if output.len() < needed_len {
                output.push(elems[idx]);
            }
        }
        idx = (idx + 1) % 4;
    }

    // eprintln!("Pattern: {:?}", output);
    output
}

fn fft_phase(values: &mut [i64], offset: usize) {
    let len = values.len();
    for i in offset..len {
        // eprintln!("On element {}/{}", i, len);
        let pattern = get_pattern(len, i);
        let next: i64 = values[i..]
            .iter()
            .zip(&pattern[i..])
            .map(|(i, p)| i * p)
            .sum();
        values[i] = next.abs() % 10;
    }
}

fn read_str(input: &str) -> Vec<i64> {
    input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).expect("invalid digit") as i64)
        .collect()
}

fn read_input(filename: &str) -> Vec<i64> {
    let raw = read_to_string(filename).unwrap();

    read_str(&raw)
}

fn flawed_frequency_transmission(input: &mut [i64], n: usize, offset: usize) {
    eprintln!("Length: {}", input.len());
    for _i in 0..n {
        // eprintln!("Iteration {}/{}", i, n);
        // eprintln!("Iteration #{} (input): {:?}", i, input);
        fft_phase(input, offset);
        // eprintln!("Iteration #{} (output): {:?}", i, input);
    }
}

fn fft_to_string(input: &[i64]) -> String {
    let mut output = String::new();
    for c in input {
        output.push_str(&c.to_string());
    }
    output
}

fn print_fft(input: &[i64]) {
    println!("{}", fft_to_string(input));
}

fn get_needed_indices(len_vec: usize, slice_start: usize, slice_len: usize) -> Vec<usize> {
    let mut used_idx: HashSet<usize> = HashSet::new();
    let mut to_check = VecDeque::new();

    for idx in slice_start..slice_start + slice_len {
        to_check.push_back(idx);
    }
    while to_check.len() > 0 {
        let idx = to_check.pop_front().unwrap();

        used_idx.insert(idx);

        let pattern = get_pattern(len_vec, idx);
        for idx in pattern
            .iter()
            .enumerate()
            .filter(|(_, factor)| **factor != 0)
            .map(|(idx, _)| idx)
            .filter(|idx| !used_idx.contains(idx))
        {
            to_check.push_back(idx);
        }
    }

    let mut retval: Vec<usize> = used_idx.iter().map(|i| *i).collect();
    retval.sort();
    retval
}

/*
 * fn fft_slice(input: &Vec<i64>, num_iterations: usize, slice_start: usize, slice_len: usize) -> Vec<i64>
 * {
 *     // TODO calculate which elements from the input we need
 *     input
 * }
 */

fn fft_repeated(input: &[i64], repetitions: usize, num_iterations: usize) -> String {
    let mut input = input.repeat(repetitions);
    let offset: usize = fft_to_string(&input[..7])
        .parse()
        .expect("Could not determine offset.");
    let len = input.len();

    if offset < len / 2 {
        panic!("Offset is in the first half of the array.");
    }
    for _ in 0..num_iterations {
        let mut sum = 0;
        for i in (offset..input.len()).rev()
        {
            sum += input[i];
            input[i] = sum.abs() % 10;
        }
    }
    // eprintln!("Offset/length: {}/{}", offset, input.len());
    fft_to_string(&input[offset..offset + 8])
}

fn main() {
    let num_iterations = 100;
    if true {
        let mut input = read_input("input.txt");
        flawed_frequency_transmission(&mut input[..], num_iterations, 0);
        // println!("After {} iterations of FFT:", num_iterations);
        print_fft(&input[..8]);
    }
    // part 2
    if false {
        for i in 1..11 {
            let mut input = read_input("input.txt").repeat(i);
            flawed_frequency_transmission(&mut input[..], num_iterations, 0);
            print_fft(&input[..8]);
        }
    }
    if false {
        let input = read_input("input.txt");
        let offset: usize = fft_to_string(&input[..7])
            .parse()
            .expect("Could not determine offset.");
        println!(
            "Number of indices needed: {}",
            get_needed_indices(input.len() * 10000, offset, 7).len()
        );
    }
    if true {
        let input = read_input("input.txt");
        println!("{}", fft_repeated(&input, 10000, 100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns() {
        assert_eq!(get_pattern(10, 0), vec![1, 0, -1, 0, 1, 0, -1, 0, 1, 0]);
        assert_eq!(get_pattern(10, 1), vec![0, 1, 1, 0, 0, -1, -1, 0, 0, 1]);
        assert_eq!(get_pattern(10, 2), vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1]);
    }

    #[test]
    fn example_01() {
        let mut input = read_str("80871224585914546619083218645595");
        let num_iterations = 100;
        flawed_frequency_transmission(&mut input[..], num_iterations, 0);
        assert!(fft_to_string(&input).starts_with("24176176"));
    }

    #[test]
    fn example_02() {
        let mut input = read_str("19617804207202209144916044189917");
        let num_iterations = 100;
        flawed_frequency_transmission(&mut input[..], num_iterations, 0);
        assert!(fft_to_string(&input).starts_with("73745418"));
    }

    #[test]
    fn example_03() {
        let mut input = read_str("69317163492948606335995924319873");
        let num_iterations = 100;
        flawed_frequency_transmission(&mut input[..], num_iterations, 0);
        assert!(fft_to_string(&input).starts_with("52432133"));
    }

    #[test]
    fn example_repeated_01() {
        let mut input = read_str("03036732577212944063491565474664");
        assert_eq!(fft_repeated(&mut input[..], 10000, 100), "84462026");
    }

    #[test]
    fn example_repeated_02() {
        let mut input = read_str("02935109699940807407585447034323");
        assert_eq!(fft_repeated(&mut input[..], 10000, 100), "78725270");
    }

    #[test]
    fn example_repeated_03() {
        let mut input = read_str("03081770884921959731165446850517");
        assert_eq!(fft_repeated(&mut input[..], 10000, 100), "53553731");
    }
}
