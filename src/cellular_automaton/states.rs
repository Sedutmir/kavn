use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn invert(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Signal(bool), // Общее для всех состояний событие прихода/отсутствия сигнала
    NotOut, // Специальное событие для конфлюентных состояний, обозначающее отсутствие сообщения с другими клетками
    Break,  // Общее для всех состояний событие, обозначающее разрушение клетки
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum State {
    Unexcitable,
    Sensitive { value: u8 },
    NormalTransmitting { dir: Direction, signal: bool },
    SpecialTransmitting { dir: Direction, signal: bool },
    Confluence { new: bool, old: bool },
}

impl State {
    pub fn next(self, event: Event) -> State {
        use Direction::*;
        use Event::*;
        use State::*;

        let next_state = match (self, event) {
            (Unexcitable, Signal(true)) => Sensitive { value: 1u8 },

            (Sensitive { value }, Signal(signal)) => Sensitive {
                value: (value << 1) + if signal { 1 } else { 0 },
            },

            (NormalTransmitting { dir, .. }, Signal(signal)) => NormalTransmitting { dir, signal },

            (SpecialTransmitting { dir, .. }, Signal(signal)) => {
                SpecialTransmitting { dir, signal }
            }

            (Confluence { .. }, NotOut) => Confluence {
                new: false,
                old: false,
            },

            (Confluence { new, .. }, Signal(signal)) => Confluence {
                new: signal,
                old: new,
            },

            (NormalTransmitting { .. } | SpecialTransmitting { .. } | Confluence { .. }, Break) => {
                Unexcitable
            }

            (state, event) => {
                panic!(
                    "Ошибка! Похоже в функцию обработки изменений состояний попала неверная пара Состояние-Событие.\
                     Стоит посмотреть в области в строки {} в файле {}, должна ли быть там обработка пары \
                     {:?} - {:?} или как туда такая пара могла попасть.",
                    line!(),
                    file!(),
                    state,
                    event
                )
            }
        };

        // Обработка того, что чувствительная клетка после события достигла точки перехода
        match next_state {
            // Обычные транспортные состояния
            Sensitive { value: 0b000_10000 } => NormalTransmitting {
                dir: Right,
                signal: false,
            },

            Sensitive { value: 0b000_10001 } => NormalTransmitting {
                dir: Up,
                signal: false,
            },

            Sensitive { value: 0b0000_1001 } => NormalTransmitting {
                dir: Left,
                signal: false,
            },

            Sensitive { value: 0b0000_1010 } => NormalTransmitting {
                dir: Down,
                signal: false,
            },

            // Специальные транспортные состояния
            Sensitive { value: 0b0000_1011 } => SpecialTransmitting {
                dir: Right,
                signal: false,
            },

            Sensitive { value: 0b0000_1100 } => SpecialTransmitting {
                dir: Up,
                signal: false,
            },

            Sensitive { value: 0b0000_1101 } => SpecialTransmitting {
                dir: Left,
                signal: false,
            },

            Sensitive { value: 0b0000_1110 } => SpecialTransmitting {
                dir: Down,
                signal: false,
            },

            // Конфлюентные состояния
            Sensitive { value: 0b0000_1111 } => Confluence {
                new: false,
                old: false,
            },

            // Не чувствительные состояния на входе
            state => state,
        }
    }

    pub fn get_event(&self, neighborhood: [&State; 4]) -> Option<Event> {
        use Direction::*;
        use Event::*;
        use State::*;

        // Добавочные направления противоположны реальному положению соседа
        let mut neighborhood_iter = neighborhood.into_iter().zip([Down, Right, Up, Left]);

        match self {
            Unexcitable => {
                if neighborhood_iter.any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: true }
                    | SpecialTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Signal(true))
                } else {
                    None
                }
            }

            // Хотя код и похож на ветку Unexcitable, но имеет одно критическое отличие
            // Конечно можно было впихнуть и в одну ветку, но я решил оставить такой вариант
            Sensitive { .. } => {
                if neighborhood_iter.any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: true }
                    | SpecialTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Signal(true))
                } else {
                    Some(Signal(false))
                }
            }

            NormalTransmitting { dir, .. } => {
                let dr = dir;

                if neighborhood_iter.clone().any(|(state, d)| match state {
                    SpecialTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Break)
                } else if neighborhood_iter.any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: true } => *dir == d && dr.invert() != *dir,

                    Confluence { old: true, .. } => {
                        if dir.invert() == d {
                            false
                        } else {
                            true
                        }
                    }

                    _ => false,
                }) {
                    Some(Signal(true))
                } else {
                    Some(Signal(false))
                }
            }

            SpecialTransmitting { dir, .. } => {
                let dr = dir;

                if neighborhood_iter.clone().any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Break)
                } else if neighborhood_iter.any(|(state, d)| match state {
                    SpecialTransmitting { dir, signal: true } => *dir == d && dr.invert() != *dir,

                    Confluence { old: true, .. } => {
                        if dir.invert() == d {
                            false
                        } else {
                            true
                        }
                    }

                    _ => false,
                }) {
                    Some(Signal(true))
                } else {
                    Some(Signal(false))
                }
            }

            Confluence { .. } => {
                if neighborhood_iter.clone().any(|(state, d)| match state {
                    SpecialTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Break)
                } else if neighborhood_iter.clone().any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: true } => *dir == d,

                    _ => false,
                }) {
                    Some(Signal(true))
                } else if neighborhood_iter.any(|(state, d)| match state {
                    NormalTransmitting { dir, signal: false } => *dir == d,

                    _ => false,
                }) {
                    Some(Signal(false))
                } else {
                    Some(NotOut)
                }
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Unexcitable
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            State::Unexcitable => "Невозбудимое",
            State::Sensitive { .. } => "Чувствительное",
            State::NormalTransmitting { .. } => "Обычное передающее",
            State::SpecialTransmitting { .. } => "Специальное передающее",
            State::Confluence { .. } => "Конфлюентное",
        };

        f.pad(s)
    }
}
