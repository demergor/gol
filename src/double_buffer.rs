use std::mem::swap;

use rand::distr::{Distribution, weighted::WeightedIndex};

pub struct DoubleBuffer {
    front: Vec<State>,
    back: Vec<State>,

    width: u16,
    height: u16,
}

impl DoubleBuffer {
    pub fn new(width: u16, height: u16) -> DoubleBuffer {
        const WEIGHTS: [i32; 4] = [5, 0, 0, 95];
        let cell_count = (width * height) as usize;

        let mut rng = rand::rng();
        let dist = WeightedIndex::new(WEIGHTS).unwrap();
        let mut front = Vec::with_capacity(cell_count);

        for _ in 0..(cell_count) {
            front.push(match dist.sample(&mut rng) {
                0 => State::Alive,
                3 => State::Dead,
                _ => unreachable!(),
            });
        }

        DoubleBuffer {
            back: vec![State::Dead; front.len()],
            front: front,

            width,
            height,
        }
    }

    pub fn swap(&mut self) {
        swap(&mut self.front, &mut self.back);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum State {
    Alive,
    Reviving,
    Dying,
    Dead,
}
