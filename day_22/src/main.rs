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

/// linear function \w modulo
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct FnLinMod {
    m: i64,
    b: i64,
    p: Option<i64>,
}

impl From<&ShuffleOperation> for FnLinMod {
    fn from(other: &ShuffleOperation) -> FnLinMod {
        use ShuffleOperation::*;
        match *other {
            DealIntoNewStack => FnLinMod {
                m: -1,
                b: -1,
                p: None,
            },
            CutN(n) => FnLinMod {
                m: 1,
                b: -n,
                p: None,
            },
            DealWithIncrement(n) => FnLinMod {
                m: n as i64,
                b: 0,
                p: None,
            },
        }
    }
}

fn modulo(x: i128, p: i128) -> i128 {
    assert!(p > 0);
    let result = x % p;

    if result < 0 {
        result + p
    } else {
        result
    }
}

/// compute x^e mod p
fn pow_mod_pos(x: i64, e: i64, p: i64) -> i64 {
    let x = x as i128;
    let p = p as i128;

    assert!(e >= 0, "Exponent needs to be positive");

    match e {
        0 => 1,
        1 => x as i64,
        _ if e % 2 == 0 => pow_mod_pos(modulo(x * x, p) as i64, e / 2, p as i64),
        _ if e % 2 == 1 => modulo(
            pow_mod_pos(modulo(x * x, p) as i64, e / 2, p as i64) as i128 * x,
            p,
        ) as i64,
        _ => {
            panic! {"Cannot happen"}
        }
    }
}

fn get_inv(x: i64, p: i64) -> i64 {
    // use fermats little theorem
    pow_mod_pos(x, p - 2, p)
}

impl FnLinMod {
    fn specify_p(&mut self, p: i64) {
        assert!(p > 0, "p needs to be positive!");
        self.p = Some(p);
        self.m %= p;
        self.b %= p;
    }

    /// apply self before other
    fn before(&self, other: &Self) -> Self {
        let p = self
            .p
            .expect("p not specified for linear modulo operation!");
        let p = p as i128;

        let m = (other.m as i128 * self.m as i128) % p;

        let b = (other.m as i128 * self.b as i128 + other.b as i128) % p;

        let m = m as i64;
        let b = b as i64;
        let p = p as i64;

        FnLinMod {
            m: m,
            b: b,
            p: Some(p),
        }
    }

    fn get_forward(&self, x: i64) -> i64 {
        let x = x as i128;
        let p = self
            .p
            .expect("p not specified for linear modulo operation!");
        let p = p as i128;

        let m = self.m as i128;
        let b = self.b as i128;

        let result = modulo(m * x + b, p);
        result as i64
    }

    fn get_backward(&self, tgt: i64) -> i64 {
        let tgt = tgt as i128;
        let p = self
            .p
            .expect("p not specified for linear modulo operation!");
        let inv_m = get_inv(self.m, p);

        let p = p as i128;
        let b = self.b as i128;

        let target = modulo(tgt - b, p);
        assert!(target >= 0);

        let result = modulo(inv_m as i128 * target, p);
        result as i64
    }

    /// Combine a vector of operations into a single linear modulo operation
    fn combine(vec: &Vec<ShuffleOperation>, p: i64) -> Self {
        let mut v_iter = vec.iter();

        let mut op: FnLinMod = v_iter.next().expect("Vector is empty!").into();
        op.specify_p(p);

        for v in v_iter {
            op = op.before(&v.into());
        }
        op
    }

    /// Apply self e times
    fn pow_apply(&self, e: usize) -> Self {
        match e {
            0 => FnLinMod {
                m: 1,
                b: 0,
                p: self.p,
            },
            1 => self.clone(),
            _ if e % 2 == 0 => self.before(self).pow_apply(e / 2),
            _ if e % 2 == 1 => self.before(self).pow_apply(e / 2).before(self),
            _ => {
                panic! {"Cannot happen"}
            }
        }
    }
}

impl ShuffleOperation {
    fn load_stack(filename: &str) -> Vec<ShuffleOperation> {
        let lines = read_to_string(filename).expect("Could not read input file.");
        ShuffleOperation::new_stack(&lines)
    }

    fn new_stack(raw: &str) -> Vec<ShuffleOperation> {
        let mut ops: Vec<ShuffleOperation> = Vec::new();

        for line in raw.lines() {
            ops.push(line.parse().expect("Could not parse operation."));
        }

        ops
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

                if idx < len - n {
                    n + idx
                } else {
                    idx + n - len
                }
            }
            DealWithIncrement(n) => {
                let mut num_iterations = 0;

                while (num_iterations * len + idx) % n != 0 {
                    num_iterations += 1;
                }
                // let num_iterations = idx % n;
                let rev_idx = (num_iterations as u128 * len as u128 + idx as u128) / n as u128;
                /*
                 * eprintln!(
                 *     "idx: {} / n: {} / num_iterations: {} / reverse index: {}",
                 *     idx, n, num_iterations, rev_idx
                 * );
                 */
                rev_idx as usize
            }
        }
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
            // let old = idx;
            idx = op.rev_shuffle_single(idx, len);
            // eprintln!("{:?} transformed: {} -> {}", op, old, idx);
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

    let vec_len = 10007;
    // part A
    {
        let vec: Vec<usize> = (0..vec_len).collect();
        let result = ShuffleOperation::apply(&stack, vec);
        println!(
            "Position of 2019: {}",
            result
                .iter()
                .position(|e| *e == 2019)
                .expect("Card 2019 missing!")
        );

        let f = FnLinMod::combine(&stack, vec_len as i64);
        println!("Position 2019 via FnLinMod: {}", f.get_forward(2019));
    }
    if false {
        let vec_len = 10037;
        // let stack_rev = ShuffleOperation::reverse(&stack);
        let mut vec: Vec<usize> = (0..vec_len).collect();
        let orig = vec.clone();

        let mut idx = 0;

        loop {
            vec = ShuffleOperation::apply(&stack, vec);

            idx += 1;
            if vec == orig {
                break;
            }
        }
        println!("Both vectors are the same after {} iterations.", idx);
    }
    // part B
    {
        let len: usize = 119315717514047;
        let shuffle_times: usize = 101741582076661;

        // // go opposite direction
        // let shuffle_times = len - shuffle_times;
        // let stack = ShuffleOperation::reverse(&stack);

        let mut f = FnLinMod::combine(&stack, len as i64);

        f = f.pow_apply(shuffle_times);

        eprintln!("\rCalculating backward path..");
        let result = f.get_backward(2020);
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
        test_reverse(&ops, &result, false);
    }

    #[test]
    fn example_02() {
        let ops = ShuffleOperation::load_stack("example_02.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result, false);
    }

    #[test]
    fn example_03() {
        let ops = ShuffleOperation::load_stack("example_03.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result, false);
    }

    #[test]
    fn example_04() {
        let ops = ShuffleOperation::load_stack("example_04.txt");
        let vec: Vec<usize> = (0..10).collect();
        let result: Vec<usize> = vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6];

        assert_eq!(result, ShuffleOperation::apply(&ops, vec));
        test_reverse(&ops, &result, false);
    }

    #[test]
    fn test_reversing_operation() {
        let stack = ShuffleOperation::load_stack("input.txt");

        let len = 10007;
        let result = {
            let vec: Vec<usize> = (0..len).collect();
            ShuffleOperation::apply(&stack, vec)
        };

        test_reverse(&stack, &result, true);
    }

    fn test_reverse(ops: &Vec<ShuffleOperation>, result: &Vec<usize>, p_is_prime: bool) {
        let rev = ShuffleOperation::reverse(ops);
        let single = FnLinMod::combine(ops, result.len() as i64);

        for (idx, item) in result.iter().enumerate() {
            println!("Checking index: {}", idx);
            assert_eq!(
                *item,
                ShuffleOperation::rev_apply_single(&rev, idx, result.len())
            );
            if p_is_prime {
                // backward path requires p to be prime
                assert_eq!(*item, single.get_backward(idx as i64) as usize);
            }
        }
    }

    #[test]
    fn test_cut_n() {
        let deck: Vec<usize> = (0..11).collect();
        for n in 0..10 {
            let ops = vec![ShuffleOperation::CutN(n)];

            let result = ShuffleOperation::apply(&ops, deck.clone());

            println!("{:?}", result);

            test_reverse(&ops, &result, true);
        }
    }

    #[test]
    fn test_exp() {
        assert_eq!(pow_mod_pos(2, 0, 10007), 1, "Failed for 2^0");
        assert_eq!(pow_mod_pos(2, 1, 10007), 2, "Failed for 2^1");
        assert_eq!(pow_mod_pos(2, 2, 10007), 4, "Failed for 2^2");
        assert_eq!(pow_mod_pos(2, 3, 10007), 8, "Failed for 2^3");
        assert_eq!(pow_mod_pos(2, 4, 10007), 16, "Failed for 2^4");
        assert_eq!(pow_mod_pos(2, 5, 10007), 32, "Failed for 2^5");
        assert_eq!(pow_mod_pos(2, 6, 10007), 64, "Failed for 2^6");
        assert_eq!(pow_mod_pos(2, 9, 10007), 512, "Failed for 2^0");
        assert_eq!(pow_mod_pos(2, 9, 3), 2);
    }

    #[test]
    fn test_inv() {
        let prime: i128 = 10007;
        let to_test: Vec<i128> = vec![23, 59, 29, 9458, 478];
        for num in to_test.iter() {
            assert_eq!(
                modulo(get_inv(*num as i64, prime as i64) as i128 * num, prime),
                1
            );
        }
    }

    #[test]
    fn test_fnlinmod_pow() {
        let prime: i64 = 119315717514047;
        let stack = ShuffleOperation::load_stack("input.txt");
        let comb = FnLinMod::combine(&stack, prime);

        let num_iterations: Vec<usize> = vec![2847, 284, 840, 239, 295, 109, 11234, 959];
        let check_idx: Vec<i64> = vec![2019, 2020, 2021, 982, 58589, 23450, 30509, 85676];

        for num in num_iterations.iter() {
            let mut manual = comb; // first application here
            for _ in 1..*num {
                manual = manual.before(&comb);
            }

            let auto = comb.pow_apply(*num);

            assert_eq!(auto, manual, "pow_apply not working");

            for idx in check_idx.iter()
            {
                let forward = auto.get_forward(*idx);
                assert_eq!(*idx, auto.get_backward(forward));
            }
        }
    }
}
