use std::collections::HashSet;
use std::fs::read_to_string;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum ShuffleOperation {
    DealIntoNewStack,
    CutN(i64),
    DealWithIncrement(usize),
}

#[derive(Debug)]
struct ParseShuffleOperationError(String);

impl ShuffleOperation {
    fn load_stack(filename: &str) -> Vec<ShuffleOperation> {
        let lines = read_to_string(filename).expect("Could not read input file.");
        ShuffleOperation::new_stack(&lines)
    }

    fn new_stack(raw: &str) -> Vec<ShuffleOperation> {
        let mut ops: Vec<ShuffleOperation> = Vec::new();

        for line in raw.lines() {
            eprintln!("{}", line);
            ops.push(line.parse().expect("Could not parse operation."));
        }

        ops
    }

    /// Determine where a single index ends up during shuffling
    fn rev_shuffle_single(&self, idx: usize, len: usize) -> usize {
        use ShuffleOperation::*;

        match *self {
            DealIntoNewStack => len - 1 - idx,
            CutN(n) => {
                let n = if n < 0 {
                    len - (-n) as usize
                } else {
                    n as usize
                };

                if idx < n {
                    n + idx
                } else {
                    idx - n
                }
            }
            DealWithIncrement(n) => {
                let num_iterations = idx % n;

                (num_iterations * len + idx) / n
            }
        }
    }

    fn shuffle(&self, mut stack: Vec<usize>) -> Vec<usize> {
        use ShuffleOperation::*;

        match self {
            DealIntoNewStack => {
                stack.reverse();
            }
            CutN(n) => {
                let n = if *n < 0 {
                    stack.len() - (-*n) as usize
                } else {
                    *n as usize
                };
                let (front, back) = stack.split_at(n);
                stack = [back, front].concat();
            }
            DealWithIncrement(n) => {
                let mut new: Vec<usize> = vec![0; stack.len()];

                for (i, e) in stack.iter().enumerate() {
                    new[i * n % stack.len()] = *e;
                }
                stack = new;
            }
        }
        stack
    }

    fn apply(ops: &Vec<ShuffleOperation>, mut vec: Vec<usize>) -> Vec<usize> {
        for op in ops.iter() {
            vec = op.shuffle(vec);
        }
        vec
    }

    fn rev_apply_single(ops: &Vec<ShuffleOperation>, idx: usize, len: usize) -> usize {
        let mut idx = idx;
        for op in ops.iter() {
            idx = op.rev_shuffle_single(idx, len);
        }
        idx
    }

    fn reverse(ops: &Vec<ShuffleOperation>) -> Vec<ShuffleOperation> {
        let mut vec = ops.clone();
        vec.reverse();
        vec
    }
}

impl FromStr for ShuffleOperation {
    type Err = ParseShuffleOperationError;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        use ShuffleOperation::*;

        let str_cut = "cut ";
        let str_dlw = "deal with increment ";

        if op.starts_with("deal into new stack") {
            Ok(DealIntoNewStack)
        } else if op.starts_with(str_cut) {
            let n: i64 = match op.trim_start_matches(str_cut).parse() {
                Ok(n) => n,
                Err(_) => {
                    return Err(ParseShuffleOperationError(String::from(
                        "Invalid number for Cut.",
                    )))
                }
            };
            Ok(CutN(n))
        } else if op.starts_with(str_dlw) {
            let n: usize = match op.trim_start_matches(str_dlw).parse() {
                Ok(n) => n,
                Err(_) => {
                    return Err(ParseShuffleOperationError(String::from(
                        "Invalid number for DealWithIncrement.",
                    )))
                }
            };
            Ok(DealWithIncrement(n))
        } else {
            Err(ParseShuffleOperationError(String::from(
                "Could not parse operation name.",
            )))
        }
    }
}

fn main() {
    let stack = ShuffleOperation::load_stack(
        &std::env::args()
            .skip(1)
            .next()
            .expect("No filename supplied."),
    );
    // part A
    {
        let vec: Vec<usize> = (0..10007).collect();
        let result = ShuffleOperation::apply(&stack, vec);
        println!(
            "Position of 2019: {}",
            result
                .iter()
                .position(|e| *e == 2019)
                .expect("Card 2019 missing!")
        );
    }
    // part B
    {
        let len: usize = 119315717514047;
        let shuffle_times: usize = 101741582076661;
        let mut result = 2020;

        let mut seen: HashSet<usize> = HashSet::new();
        let mut order: Vec<usize> = Vec::new();

        let stack = ShuffleOperation::reverse(&stack);

        for i in 0..shuffle_times {
            if i % (shuffle_times / 1000) == 0 {
                eprint!("\rProgress {}‰", i * 1000 / shuffle_times);
            }
            result = ShuffleOperation::rev_apply_single(&stack, result, len);
            if seen.contains(&result) {
                result = order[shuffle_times % seen.len()];
                break;
            } else {
                order.push(result);
                seen.insert(result);
            }
        }
        println!("\rCard in position 2020: {}", result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_01() {
        let ops = ShuffleOperation::load_stack("example_01.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
    }

    #[test]
    fn example_02() {
        let ops = ShuffleOperation::load_stack("example_02.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result);
    }

    #[test]
    fn example_03() {
        let ops = ShuffleOperation::load_stack("example_03.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result);
    }

    #[test]
    fn example_04() {
        let ops = ShuffleOperation::load_stack("example_04.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result);
    }

    #[test]
    fn test_reversing_operation() {
        let stack = ShuffleOperation::load_stack("input.txt");

        let len = 10007;
        let result = {
            let vec: Vec<usize> = (0..len).collect();
            ShuffleOperation::apply(&stack, vec)
        };

        test_reverse(&stack, &result);
    }

    fn test_reverse(ops: &Vec<ShuffleOperation>, result: &Vec<usize>)
    {
        let rev = ShuffleOperation::reverse(ops);

        for (idx, item) in result.iter().enumerate()
        {
            println!("Checking index: {}", idx);
            assert_eq!(*item, ShuffleOperation::rev_apply_single(&rev, idx, result.len()));
        }
    }
}
