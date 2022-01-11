use macroquad::prelude::{draw_circle, draw_line, Color};
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_height, screen_width};
use std::rc::Rc;
use std::sync::RwLock;

use crate::cellular_automaton::*;

pub struct CellularAutomatonPainter {
    pub cellular_automaton: Rc<RwLock<CellularAutomation>>,
    pub offset: (f32, f32),
    pub cell_size: (f32, f32),
}

impl CellularAutomatonPainter {
    pub fn new(cellular_automaton: Rc<RwLock<CellularAutomation>>, cell_size: (f32, f32)) -> Self {
        Self {
            cellular_automaton,
            cell_size,
            offset: (0.0, 0.0),
        }
    }

    pub fn change_cell_size(&mut self, new_size: (f32, f32)) {
        let (ow, oh) = self.cell_size;
        let (nw, nh) = new_size;

        if nw < 8.0 || nh < 8.0 {
            return;
        }

        let (kw, kh) = (nw / ow, nh / oh);

        self.cell_size = new_size;

        self.offset.0 *= kw;
        self.offset.1 *= kh;
    }

    fn draw_state(&self, coords: (f32, f32), state: &State) {
        match *state {
            State::Unexcitable => {}

            State::Sensitive { mut value } => {
                let mut signals: Vec<bool> = vec![];

                let mut count = 0;
                while value & 0b10000000 != 0b10000000 {
                    value = value << 1;
                    count += 1;
                }

                for _ in 0..(8 - count) {
                    value = value << 1;
                    signals.push(value & 0b10000000 == 0b10000000);
                }

                let (cx, cy) = (
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 2.0,
                );
                for (v, &k) in signals.into_iter().zip([1.0, 0.75, 0.5, 0.25].iter()) {
                    draw_circle(
                        cx,
                        cy,
                        self.cell_size.0 / 1.9 * k,
                        if v {
                            Color::new(0.0, 1.0, 0.0, 1.0)
                        } else {
                            Color::new(1.0, 1.0, 1.0, 1.0)
                        },
                    );
                    draw_circle(
                        cx,
                        cy,
                        self.cell_size.0 / 2.0 * k,
                        if v {
                            Color::new(0.0, 1.0, 0.0, 1.0)
                        } else {
                            Color::new(0.0, 0.0, 0.0, 1.0)
                        },
                    );
                }
            }

            State::NormalTransmitting {
                dir: Direction::Up,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 - self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );
            }

            State::NormalTransmitting {
                dir: Direction::Left,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 - self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );
            }

            State::NormalTransmitting {
                dir: Direction::Down,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 - self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );
            }

            State::NormalTransmitting {
                dir: Direction::Right,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 - self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 - self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 - self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 - self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 - self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );
            }

            State::SpecialTransmitting {
                dir: Direction::Up,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 - self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_circle(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 2.0 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 8.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
            }

            State::SpecialTransmitting {
                dir: Direction::Left,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 - self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_circle(
                    coords.0 + self.cell_size.0 / 2.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    self.cell_size.0 / 8.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
            }

            State::SpecialTransmitting {
                dir: Direction::Down,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 - self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 / 8.0,
                    coords.0 + self.cell_size.0 / 2.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 - self.cell_size.1 * 7.0 / 8.0,
                    self.cell_size.0 / 10.0,
                    c,
                );
                draw_circle(
                    coords.0 + self.cell_size.0 / 2.0,
                    coords.1 + self.cell_size.1 / 2.0 - self.cell_size.1 / 4.0,
                    self.cell_size.0 / 8.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
            }

            State::SpecialTransmitting {
                dir: Direction::Right,
                signal,
            } => {
                let c = if signal {
                    Color::new(0.0, 1.0, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };

                draw_line(
                    coords.0 + self.cell_size.0 - self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 - self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 - self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_line(
                    coords.0 + self.cell_size.0 - self.cell_size.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    coords.0 + self.cell_size.0 - self.cell_size.0 * 7.0 / 8.0,
                    coords.1 + self.cell_size.1 / 2.0 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 10.0,
                    c,
                );

                draw_circle(
                    coords.0 + self.cell_size.0 / 2.0 - self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 / 2.0,
                    self.cell_size.0 / 8.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
            }

            State::Confluence { new, old } => {
                draw_rectangle(
                    coords.0,
                    coords.1,
                    self.cell_size.0,
                    self.cell_size.1,
                    if old {
                        Color::new(0.0, 1.0, 0.0, 1.0)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 1.0)
                    },
                );

                draw_rectangle(
                    coords.0 + self.cell_size.0 / 4.0,
                    coords.1 + self.cell_size.1 / 4.0,
                    self.cell_size.0 / 2.0,
                    self.cell_size.1 / 2.0,
                    if new {
                        Color::new(0.0, 1.0, 0.0, 1.0)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 1.0)
                    },
                );
            }
        }
    }

    pub fn draw(&self) {
        let (cx, cy) = self.cell_size;

        let (mut x, mut y) = (0.0, 0.0);
        while x < screen_width() {
            draw_line(
                x,
                0.0,
                x,
                screen_height(),
                cx / 20.0,
                Color::new(0.0, 0.0, 0.0, 1.0),
            );
            x += cx;
        }

        while y < screen_height() {
            draw_line(
                0.0,
                y,
                screen_width(),
                y,
                cy / 20.0,
                Color::new(0.0, 0.0, 0.0, 1.0),
            );
            y += cy;
        }

        let ca = self.cellular_automaton.read().unwrap();

        for y in (-ca.center.1)..(ca.field.len() as i64 - ca.center.1) {
            for x in (-ca.center.0)..(ca.field[0].len() as i64 - ca.center.0) {
                let state = ca.get_cell((x, y));
                let (x, y) = (x as f32, y as f32);

                self.draw_state(
                    (
                        self.cell_size.0 * x + self.offset.0,
                        self.cell_size.1 * y + self.offset.1,
                    ),
                    state,
                );
            }
        }
    }
}
