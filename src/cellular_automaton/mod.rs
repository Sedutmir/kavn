mod states;
use serde::{Deserialize, Serialize};

pub use states::*;
use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct CellularAutomation {
    pub field: VecDeque<VecDeque<State>>,
    pub tact: u64,
    pub center: (i64, i64),
}

impl Clone for CellularAutomation {
    fn clone(&self) -> Self {
        let field = {
            let mut f = VecDeque::new();

            for row in &self.field {
                let mut c_row = VecDeque::new();

                for state in row {
                    c_row.push_front(state.clone());
                }

                f.push_front(c_row);
            }

            f
        };

        Self {
            tact: self.tact,
            center: self.center,
            field,
        }
    }
}

impl CellularAutomation {
    pub fn new() -> Self {
        Self {
            field: {
                let mut rows = VecDeque::with_capacity(3);

                for _ in 0..3 {
                    rows.push_back(VecDeque::from([
                        State::Unexcitable,
                        State::Unexcitable,
                        State::Unexcitable,
                    ]));
                }

                rows
            },
            center: (1, 1),
            tact: 0,
        }
    }

    pub fn get_cell(&self, coords: (i64, i64)) -> &State {
        &self.field[(self.center.1 + coords.1) as usize][(self.center.0 + coords.0) as usize]
    }

    pub fn set_cell(&mut self, coords: (i64, i64), value: State) {
        while self.center.1 + coords.1 < 2 {
            let mut new_row = VecDeque::new();
            new_row.resize_with(self.field[0].len(), Default::default);

            self.field.push_front(new_row);

            self.center.1 += 1;
        }

        while self.center.0 + coords.0 < 2 {
            self.field.iter_mut().for_each(|row| {
                row.push_front(Default::default());
            });

            self.center.0 += 1;
        }

        while (self.center.1 + coords.1) as usize >= self.field.len() - 2 {
            let mut new_row = VecDeque::new();
            new_row.resize_with(self.field[0].len(), Default::default);

            self.field.push_back(new_row);
        }

        while (self.center.0 + coords.0) as usize >= self.field[0].len() - 2 {
            self.field.iter_mut().for_each(|row| {
                row.push_back(Default::default());
            });
        }

        self.field[(self.center.1 + coords.1) as usize][(self.center.0 + coords.0) as usize] =
            value;
    }

    pub fn tact(&mut self) {
        self.tact += 1;

        let mut event_buffer = HashMap::new();

        for (y, row) in self.field.iter().enumerate() {
            if y == 0 || y == self.field.len() - 1 {
                continue;
            }

            for (x, state) in row.iter().enumerate() {
                if x == 0 || x == self.field[0].len() - 1 {
                    continue;
                }

                match state.get_event([
                    &self.field[y - 1][x],
                    &self.field[y][x - 1],
                    &self.field[y + 1][x],
                    &self.field[y][x + 1],
                ]) {
                    Some(event) => {
                        event_buffer
                            .insert((x as i64 - self.center.0, y as i64 - self.center.1), event);
                    }
                    _ => (),
                };
            }
        }

        for ((x, y), event) in event_buffer {
            self.set_cell((x, y), self.get_cell((x, y)).next(event));
        }
    }
}
