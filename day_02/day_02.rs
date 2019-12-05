fn main() {
    // last minute replacements
    run_1202(12, 2);

    println!("Left in position 0: {}", run_1202(12, 2));

    let target_output = 19690720;

    for noun in 0..100
    {
        for verb in 0..100
        {
            if run_1202(noun, verb) == target_output
            {
                println!("Found: {}", noun * 100 + verb);
                break;
            }
        }
    }
}

fn run_1202(noun: usize, verb: usize) -> usize {
    let mut tape: [usize; 133] = [
        1, 0, 0, 3, 1, 1, 2, 3, 1, 3, 4, 3, 1, 5, 0, 3, 2, 6, 1, 19, 1, 5, 19, 23, 2, 9, 23, 27, 1,
        6, 27, 31, 1, 31, 9, 35, 2, 35, 10, 39, 1, 5, 39, 43, 2, 43, 9, 47, 1, 5, 47, 51, 1, 51, 5,
        55, 1, 55, 9, 59, 2, 59, 13, 63, 1, 63, 9, 67, 1, 9, 67, 71, 2, 71, 10, 75, 1, 75, 6, 79,
        2, 10, 79, 83, 1, 5, 83, 87, 2, 87, 10, 91, 1, 91, 5, 95, 1, 6, 95, 99, 2, 99, 13, 103, 1,
        103, 6, 107, 1, 107, 5, 111, 2, 6, 111, 115, 1, 115, 13, 119, 1, 119, 2, 123, 1, 5, 123, 0,
        99, 2, 0, 14, 0,
    ];

    tape[1] = noun;
    tape[2] = verb;

    let mut pos: usize = 0;

    loop {
        let opcode: usize = tape[pos];
        pos = match opcode {
            1 => {
                let (input1, input2) = get_input(&tape, pos);
                let idx_target = tape[pos + 3];
                tape[idx_target] = input1 + input2;
                pos + 4
            }
            2 => {
                let (input1, input2) = get_input(&tape, pos);
                let idx_target = tape[pos + 3];
                tape[idx_target] = input1 * input2;
                pos + 4
            }
            99 => break,
            _ => {
                println!("Encountered invalid Opcode {}!", opcode);
                break;
            }
        }
    }
    tape[0]
}

fn get_input(tape: &[usize], pos: usize) -> (usize, usize) {
    (tape[tape[pos + 1]], tape[tape[pos + 2]])
}
