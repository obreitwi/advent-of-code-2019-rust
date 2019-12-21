use std::fs::read_to_string;

#[derive(Debug)]
struct Layer {
    height: usize,
    width: usize,
    data: Vec<i64>,
}

#[derive(Debug)]
struct Image {
    height: usize,
    width: usize,
    layers: Vec<Layer>,
}

enum Color {
    Black,
    White,
    Transparent,
}

impl Color {
    fn from_i64(i: i64) -> Color {
        use Color::*;

        match i {
            0 => Black,
            1 => White,
            2 => Transparent,
            _ => panic!("Invalid digit for color!"),
        }
    }

    fn to_i64(&self) -> i64 {
        use Color::*;
        match self {
            Black => 0,
            White => 1,
            Transparent => 2,
        }
    }

    fn to_char(&self) -> char {
        use Color::*;
        match self {
            Black => 'X',
            White => '.',
            Transparent => ' ',
        }
    }
}

impl Image {
    fn new(raw: &[char], width: usize, height: usize) -> Image {
        let step = height * width;
        let mut layers = Vec::new();
        println!("raw.len(): {} / step: {}", raw.len(), step);
        assert!(raw.len() % step == 0);
        let num_layers = raw.len() / step;

        for i in 0..num_layers {
            layers.push(Layer::new(
                &raw[(i * step)..((i + 1) * step)],
                width,
                height,
            ));
        }
        Image {
            height,
            width,
            layers,
        }
    }

    fn get(&self, x: usize, y: usize) -> char {
        use Color::*;
        let mut color = Transparent;
        for l in self.layers.iter() {
            color = Color::from_i64(l.get(x, y));

            if let Transparent = color {
                continue;
            }
            else {
                break;
            }
        }
        color.to_char()
    }

    fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}", self.get(x, y));
            }
            println!();
        }
    }
}

impl Layer {
    fn new(raw: &[char], width: usize, height: usize) -> Layer {
        let data: Vec<i64> = raw
            .iter()
            .map(|c| c.to_digit(10).expect("Invalid char converted to integer") as i64)
            .collect();

        assert!(data.len() == height * width);

        Layer {
            width,
            height,
            data,
        }
    }

    fn count_digit(&self, digit: i64) -> usize {
        self.data.iter().filter(|n| **n == digit).count()
    }

    fn get(&self, x: usize, y: usize) -> i64 {
        self.data[y * self.width + x]
    }
}

fn argmin<R: PartialOrd, T: Iterator<Item = R>>(iter: &mut T) -> usize {
    let mut arg = 0;
    let mut min = iter.next().expect("Iterator cannot be empty!");

    for (i, elem) in iter.enumerate() {
        if elem < min {
            arg = i + 1; // we took the first element in the beginning
            min = elem;
        }
    }
    arg
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    assert!(args.len() > 0);

    let raw: Vec<char> = read_to_string(&args[0]).unwrap().trim().chars().collect();
    let image = Image::new(&raw, 25, 6);

    let num_0_digits: Vec<usize> = image
        .layers
        .iter()
        .map(|l| l.data.iter().filter(|n| **n == 0).count())
        .collect();

    println!("{:?}", num_0_digits);

    let layer_least_0 = argmin(&mut num_0_digits.iter());

    let layer = &image.layers[layer_least_0];

    println!("Layer with least 0s: {}", layer_least_0);
    println!("Number of 1s: {}", layer.count_digit(1));
    println!("Number of 2s: {}", layer.count_digit(2));
    println!(
        "Multiplication: {}",
        layer.count_digit(1) * layer.count_digit(2)
    );
    image.print();
}
