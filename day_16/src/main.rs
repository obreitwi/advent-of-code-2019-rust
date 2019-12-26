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

fn fft_phase(input: Vec<i64>, patterns: &Vec<Vec<i64>>) -> Vec<i64> {
    let mut output = Vec::with_capacity(input.len());
    let len = input.len();
    for i in 0..len {
        // eprintln!("On element {}/{}", i, len);
        let pattern = &patterns[i];
        let next: i64 = input.iter().zip(pattern).map(|(i, p)| i * p).sum();
        output.push(next.abs() % 10);
    }
    output
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

fn flawed_frequency_transmission(input: Vec<i64>, n: usize) -> Vec<i64> {
    let mut input = input;
    let patterns: Vec<Vec<i64>> = (0..input.len())
        .map(|i| get_pattern(input.len(), i))
        .collect();
    for _i in 0..n {
        // eprintln!("Iteration {}/{}", i, n);
        // eprintln!("Iteration #{} (input): {:?}", i, input);
        input = fft_phase(input, &patterns);
        // eprintln!("Iteration #{} (output): {:?}", i, input);
    }
    input
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

fn fft_repeated(input: &Vec<i64>, repetitions: usize, num_iterations: usize) -> String {
    let input = input.repeat(repetitions);
    let offset: usize = fft_to_string(&input[..7])
        .parse()
        .expect("Could not determine offset.");
    let after = flawed_frequency_transmission(input, num_iterations);
    fft_to_string(&after[offset..offset + 8])
}

fn main() {
    let num_iterations = 100;
    {
        let input = read_input("input.txt");
        let after = flawed_frequency_transmission(input, num_iterations);
        // println!("After {} iterations of FFT:", num_iterations);
        print_fft(&after[..8]);
    }
    // part 2
    {
        for i in 1..11 {
            let input = read_input("input.txt").repeat(i);
            let after = flawed_frequency_transmission(input, num_iterations);
            print_fft(&after[..8]);
        }
    }
    if false {
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
        let input = read_str("80871224585914546619083218645595");
        let num_iterations = 100;
        let after = flawed_frequency_transmission(input, num_iterations);
        assert!(fft_to_string(&after).starts_with("24176176"));
    }

    #[test]
    fn example_02() {
        let input = read_str("19617804207202209144916044189917");
        let num_iterations = 100;
        let after = flawed_frequency_transmission(input, num_iterations);
        assert!(fft_to_string(&after).starts_with("73745418"));
    }

    #[test]
    fn example_03() {
        let input = read_str("69317163492948606335995924319873");
        let num_iterations = 100;
        let after = flawed_frequency_transmission(input, num_iterations);
        assert!(fft_to_string(&after).starts_with("52432133"));
    }

    #[test]
    fn example_repeated_01() {
        let input = read_str("03036732577212944063491565474664");
        assert_eq!(fft_repeated(&input, 10000, 100), "84462026");
    }

    #[test]
    fn example_repeated_02() {
        let input = read_str("02935109699940807407585447034323");
        assert_eq!(fft_repeated(&input, 10000, 100), "78725270");
    }

    #[test]
    fn example_repeated_03() {
        let input = read_str("03081770884921959731165446850517");
        assert_eq!(fft_repeated(&input, 10000, 100), "53553731");
    }
}
