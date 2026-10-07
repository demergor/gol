use std::{
    cmp::{max, min},
    mem::swap,
};

use rand::distr::{Distribution, weighted::WeightedIndex};

#[derive(Clone)]
pub struct DoubleBuffer {
    front: Vec<State>,
    back: Vec<State>,

    width: u16,
    height: u16,

    dirty_tl: (usize, usize),
    dirty_br: (usize, usize),
}

impl DoubleBuffer {
    pub fn new(width: u16, height: u16) -> DoubleBuffer {
        const WEIGHTS: [i32; 4] = [50, 0, 0, 50];
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

            dirty_tl: (0, 0),
            dirty_br: (width as usize, height as usize),
        }
    }

    pub fn swap(&mut self) {
        swap(&mut self.front, &mut self.back);
    }

    pub fn update(&mut self) {
        let width = self.width as usize;
        let height = self.height as usize;
        let offsets: [i32; 8] = [
            -(width as i32) - 1,
            -(width as i32),
            -(width as i32) + 1,
            -1,
            1,
            width as i32 - 1,
            width as i32,
            width as i32 + 1,
        ];

        self.dirty_tl = (width, height);
        self.dirty_br = (0, 0);

        for y in 0..height {
            for x in 0..width {
                let cur_cell = y * width + x;
                if self.front[cur_cell] == State::Dying {
                    self.back[cur_cell] = State::Dead;

                    self.dirty_tl.0 = min(x, self.dirty_tl.0);
                    self.dirty_tl.1 = min(y, self.dirty_tl.1);
                    self.dirty_br.0 = max(x + 1, self.dirty_br.0);
                    self.dirty_br.1 = max(y + 1, self.dirty_br.1);

                    continue;
                }

                if self.front[cur_cell] == State::Reviving {
                    self.back[cur_cell] = State::Alive;

                    self.dirty_tl.0 = min(x, self.dirty_tl.0);
                    self.dirty_tl.1 = min(y, self.dirty_tl.1);
                    self.dirty_br.0 = max(x + 1, self.dirty_br.0);
                    self.dirty_br.1 = max(y + 1, self.dirty_br.1);

                    continue;
                }

                let mut live_neighbors = 0;
                for i in 0..offsets.len() {
                    if x == 0 {
                        match i {
                            0 | 3 | 5 => continue,
                            _ => (),
                        }
                    } else if x == width - 1 {
                        match i {
                            2 | 4 | 7 => continue,
                            _ => (),
                        }
                    }

                    let offset = offsets[i];
                    if offset < 0 {
                        let offset = -offset as usize;
                        if cur_cell >= offset
                            && (self.front[cur_cell - offset] == State::Alive
                                || self.front[cur_cell - offset] == State::Dying)
                        {
                            live_neighbors += 1;
                        }
                    } else {
                        let offset = offset as usize;
                        if cur_cell + offset < self.front.len()
                            && (self.front[cur_cell + offset] == State::Alive
                                || self.front[cur_cell + offset] == State::Dying)
                        {
                            live_neighbors += 1;
                        }
                    }
                }

                let new_state = match live_neighbors {
                    3 if self.front[cur_cell] == State::Dead => State::Reviving,
                    n if n < 2 => {
                        if self.front[cur_cell] == State::Alive {
                            State::Dying
                        } else {
                            State::Dead
                        }
                    }
                    n if n > 3 => {
                        if self.front[cur_cell] == State::Alive {
                            State::Dying
                        } else {
                            State::Dead
                        }
                    }
                    _ => self.front[cur_cell],
                };

                if new_state != self.front[cur_cell] {
                    self.dirty_tl.0 = min(x, self.dirty_tl.0);
                    self.dirty_tl.1 = min(y, self.dirty_tl.1);
                    self.dirty_br.0 = max(x + 1, self.dirty_br.0);
                    self.dirty_br.1 = max(y + 1, self.dirty_br.1);
                }

                self.back[cur_cell] = new_state;
            }
        }
    }

    pub fn render_string(&self) -> String {
        if self.dirty_tl.0 >= self.dirty_br.0 || self.dirty_tl.1 >= self.dirty_br.1 {
            return String::from("");
        }

        let mut buf = String::with_capacity(
            (self.dirty_br.0 - self.dirty_tl.0)
                * (self.dirty_br.1 - self.dirty_tl.1)
                * 3,
        );

        for y in self.dirty_tl.1..self.dirty_br.1 {
            buf.push_str(&format!("\x1b[{};{}H", y + 1, self.dirty_tl.0 + 1));
            for x in self.dirty_tl.0..self.dirty_br.0 {
                buf.push_str(match self.front[y * self.width as usize + x] {
                    State::Alive => "\x1b[1mO\x1b[0m",
                    State::Reviving => "\x1b[32mO\x1b[0m",
                    State::Dying => "\x1b[31mO\x1b[0m",
                    State::Dead => " ",
                });
            }
        }

        buf
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Alive,
    Reviving,
    Dying,
    Dead,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_is_stable() {
        let mut test_dbuf = DoubleBuffer::new(2, 2);
        test_dbuf.front.fill(State::Alive);
        let expected_dbuf = test_dbuf.clone();

        for _ in 0..5 {
            test_dbuf.update();
            test_dbuf.swap();
            assert_eq!(expected_dbuf.front, test_dbuf.front);
        }
    }

    #[test]
    fn oval_is_stable() {
        let mut test_dbuf = DoubleBuffer::new(4, 3);
        for i in 0..test_dbuf.front.len() {
            match i {
                1 | 2 | 4 | 7 | 9 | 10 => test_dbuf.front[i] = State::Alive,
                _ => test_dbuf.front[i] = State::Dead,
            }
        }

        let expected_dbuf = test_dbuf.clone();

        for _ in 0..5 {
            test_dbuf.update();
            test_dbuf.swap();
            assert_eq!(expected_dbuf.front, test_dbuf.front);
        }
    }

    #[test]
    fn single_doesnt_survive() {
        let mut test_dbuf = DoubleBuffer::new(3, 3);
        test_dbuf.front.fill(State::Dead);
        test_dbuf.front[4] = State::Alive;

        test_dbuf.update();
        test_dbuf.swap();
        assert_eq!(test_dbuf.front[4], State::Dying);

        test_dbuf.update();
        test_dbuf.swap();
        assert_eq!(test_dbuf.front[4], State::Dead);
    }

    #[test]
    fn dirty_area_behaves_correctly() {
        let mut test_dbuf = DoubleBuffer::new(4, 4);
        for i in 0..test_dbuf.front.len() {
            match i {
                0 | 5 | 6 | 9 | 14 => test_dbuf.front[i] = State::Alive,
                _ => test_dbuf.front[i] = State::Dead,
            }
        }

        test_dbuf.update();

        let expected_dirty_tl: (usize, usize) = (0, 0);
        let expected_dirty_br: (usize, usize) = (3, 4);

        assert_eq!(expected_dirty_tl, test_dbuf.dirty_tl);
        assert_eq!(expected_dirty_br, test_dbuf.dirty_br);
    }
}
