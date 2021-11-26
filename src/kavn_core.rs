// Из-за частого применения я решил сделать алиас.
type IVec2 = (isize, isize);

use core::panic;
use std::collections::{VecDeque, HashMap};

/// Модуль, в котором реэкспортированы все наиболее вероятные для использования моменты.
pub mod prelude {}

/// Характеризует логическую часть клеточного автомата.
/// Самая главная структура во всей программе - потому что именно она управляет всей логикой и полем.
pub struct KAVN {
    tact: u64,
    pub field: Field,
    history_points: VecDeque<(u64, Field)>,
}

impl KAVN {
    pub fn new() -> Self {
        Self {
            tact: 0,
            field: Field::new(),
            history_points: VecDeque::new(),
        }
    }

    pub fn next_tact(&mut self) {
        // Состояние поля сохраняется каждых 100 тактов, включая самое начало.
        if self.tact % 100 == 0 {
            self.history_points.push_back((self.tact, self.field.clone()));
        }

        self.field.next_tact();
    }

    fn next_tacts(&mut self, n: u64) {
        for _ in 0..n {
            self.next_tact();
        }
    }

    fn load_history_point(&mut self, tact_marker: u64) {
        while self.history_points.back().expect("Ошибка: произошла попытка откатиться к несуществующей точке истории.").0 > tact_marker {
            self.history_points.pop_back();
        }
        
        let (new_tact, new_field) = self.history_points.back().unwrap().clone();
        self.field = new_field;
        self.tact = new_tact;
    }

    pub fn to_tact(&mut self, tact: u64) {
        if tact < self.tact {
            self.load_history_point(tact);
        }

        self.next_tacts(tact - self.tact);
    }

    pub fn do_tacts(&mut self, tacts: i64) {
        self.to_tact((self.tact as i64 + tacts) as u64);
    }
}

/// Структура, являющаяся, как ни странно, полем.
/// Хранит в себе двумерный вектор и рассчитана на возможность расширения при достижением ячеками краёв.
#[derive(Clone)] // Нужно для возможности клонировать состояние поля в историю.
pub struct Field {
    center: IVec2,
    pub cells: VecDeque<VecDeque<Cell>>,
}

impl Field {
    fn new() -> Self {
        let mut cells: VecDeque<VecDeque<Cell>> = VecDeque::new();

        cells.resize(3, Default::default());
        cells.iter_mut().for_each(|row| row.resize(3, Default::default()));


        Self {
            center: (1, 1),
            cells: cells,
        }
    }

    pub fn get_cell(&self, coords: IVec2) -> &Cell {
        let (cx, cy) = self.center;
        let (x, y) = coords;

        if cx + x < 0 || cx + x > self.cells[0].len() as isize || 
           cy + y < 0 || cy + y > self.cells.len() as isize {
            panic!("Что-то пошло не так! В программе пыталось использоваться значение несуществующей ячейки.ы")
        } else {
            &self.cells[(cy + y) as usize][(cx + x) as usize]
        }
    }

    fn get_mut_cell(&mut self, coords: IVec2) -> &mut Cell {
        let (cx, cy) = self.center;
        let (x, y) = coords;

        if cx + x < 0 || cx + x > self.cells[0].len() as isize || 
           cy + y < 0 || cy + y > self.cells.len() as isize {
            panic!("Что-то пошло не так! В программе пыталось использоваться значение несуществующей ячейки.")
        } else {
            &mut self.cells[(cy + y) as usize][(cx + x) as usize]
        }
    }

    /// Указывает ячейке новое значение.
    /// Если такой ячейки ещё нет, расширяет поле до нужного размера с запасом в один ряд/столбец.
    pub fn set_cell(&mut self, coords: IVec2, value: Cell) {
        let (x, y) = coords;

        // 1 нужно для создание запаса
        while self.center.1 + y < 1 {
            self.cells.push_front(Default::default());
            
            {
                let len_x = self.cells[self.center.1 as usize].len();
                let row = self.cells.front_mut()
                      .expect("Ошибка: почему-то нет никаких строк на поле!");

                while len_x > row.len() {
                    row.push_back(Default::default());
                }
            }
            
            self.center.1 += 1;
        }

        // -2, потому что при индексе равном длине массива уже происходит выход за границы, а нужно ещё сделать запас.
        while self.center.1 + y > self.cells.len() as isize - 2 {
            self.cells.push_back(Default::default());

            {
                let len_x = self.cells[self.center.1 as usize].len();
                let row = self.cells.back_mut()
                      .expect("Ошибка: почему-то нет никаких строк на поле!");

                while len_x > row.len() {
                    row.push_back(Default::default());
                }
            }
        }

        while self.center.0 + x < 1 {
            self.cells.iter_mut().for_each(|row | row.push_front(Default::default()));
            self.center.0 += 1;
        }
        
        while self.center.0 + x > self.cells[0].len() as isize - 2 {
            self.cells.iter_mut().for_each(|row | row.push_back(Default::default()));
        }

        let cell = self.get_mut_cell(coords);

        *cell = value;
    }

    fn next_tact(&mut self) {
        // Буффер нужен для эмуляции синхронного изменения.
        let mut buffer= HashMap::new();
        self.collecting_changes(&mut buffer);
        self.handle_changes(&buffer);
    }

    fn collecting_changes(&self, buffer: &mut HashMap<IVec2, Vec<Events>>) {
        let (cx, cy) = self.center;
        
        for y in 1..(self.cells.len() - 1) {
            for x in 1..(self.cells[y].len() - 1) {

                let coords: IVec2 = (x as isize - cx, y as isize - cy);

                match self.get_cell(coords) {
                    Cell::Unexcitable => {},
                    Cell::Sensetive(_) => {
                        let val = buffer.entry(coords).or_insert(Default::default());
                        val.push(Events::Tact);
                    },
                    Cell::NormalTransmitting(dir, value) => {
                        let target = match dir {
                            Direction::Up    => (coords.0    , coords.1 - 1),
                            Direction::Right => (coords.0 + 1, coords.1    ),
                            Direction::Down  => (coords.0    , coords.1 + 1),
                            Direction::Left  => (coords.0 - 1, coords.1    ),
                        };

                        let event = match *self.get_cell(target) {
                            Cell::SpecialTransmitting(_, _) if *value => Events::Break,
                            _ => Events::Signal(*value),
                        };

                        let val = buffer.entry(target).or_insert(Default::default());
                        val.push(event);

                        if *value {
                            let val = buffer.entry(coords).or_insert(Default::default());
                            val.push(Events::SignalDrift);
                        }
                    },
                    Cell::SpecialTransmitting(dir, value) => {
                        let target = match dir {
                            Direction::Up    => (coords.0    , coords.1 - 1),
                            Direction::Right => (coords.0 + 1, coords.1    ),
                            Direction::Down  => (coords.0    , coords.1 + 1),
                            Direction::Left  => (coords.0 - 1, coords.1    ),
                        };

                        let event = match *self.get_cell(target) {
                            Cell::NormalTransmitting(_, _) | 
                            Cell::Confluence(_, _) if *value => Events::Break,
                            _ => Events::Signal(*value),
                        };

                        let val = buffer.entry(target).or_insert(Default::default());
                        val.push(event);

                        if *value {
                            let val = buffer.entry(coords).or_insert(Default::default());
                            val.push(Events::SignalDrift);
                        }
                    },
                    Cell::Confluence(_current, previous) => {
                        let locality_coords = [
                            (coords.0 + 1, coords.1    ),
                            (coords.0 - 1, coords.1    ),
                            (coords.0    , coords.1 + 1),
                            (coords.0    , coords.1 - 1),
                        ];

                        let locality = [
                            (self.get_cell(locality_coords[0]), Direction::Right),
                            (self.get_cell(locality_coords[1]), Direction::Left),
                            (self.get_cell(locality_coords[2]), Direction::Down),
                            (self.get_cell(locality_coords[3]), Direction::Up),
                        ];

                        if !locality.iter().any(|(&c, d)| {
                            match c {
                                Cell::NormalTransmitting(dir, _)  |
                                Cell::SpecialTransmitting(dir, _)   => {
                                    Some(true) == d.is_same_direction(&dir)
                                },
                                _ => false,
                            }
                        })
                        {
                            let val = buffer.entry(coords).or_insert(Default::default());
                            val.push(Events::NotOut);
                        }
                        else {
                            if *previous {
                                for (cell_coords, (cell, cell_dir)) in locality_coords.iter().zip(locality.iter()) {
                                    if match cell {
                                        Cell::NormalTransmitting(dir, _)  |
                                        Cell::SpecialTransmitting(dir, _)   => {
                                            Some(false) == cell_dir.is_same_direction(&dir)
                                        },
                                        _ => false,
                                    }
                                    {
                                        let val = buffer.entry(*cell_coords).or_insert(Default::default());
                                        val.push(Events::Signal(*previous));
                                    }
                                }
                            }

                            let val = buffer.entry(coords).or_insert(Default::default());
                            val.push(Events::SignalDrift);
                        }
                    },
                }
            }
        }
    }

    fn handle_changes(&mut self, buffer: &HashMap<IVec2, Vec<Events>>) {
        for (&coords, events) in buffer {

            let cell = self.get_mut_cell(coords);

            match cell {
                // Cell::Unexcitable => *cell = Cell::Sensetive(0b_00000001),
                Cell::Unexcitable => {
                    if events.iter().any(|e| *e == Events::Signal(true)) {
                        self.set_cell(coords, Cell::Sensetive(0b_00000001))
                    }
                },

                Cell::Sensetive(v) => {
                    
                    if events.iter().any(|e| {
                        match e {
                            Events::Signal(true) => true,
                            _ => false,
                        }
                    }) {
                        *v <<= 1;
                        *v += 1;
                    } else {
                        *v <<= 1;
                    }

                    match v {
                        0b_000_1_0000 => *cell = Cell::NormalTransmitting (Direction::Right, false),
                        0b_000_1_0001 => *cell = Cell::NormalTransmitting (Direction::Up   , false),
                        0b_0000_1_001 => *cell = Cell::NormalTransmitting (Direction::Left , false),
                        0b_0000_1_010 => *cell = Cell::NormalTransmitting (Direction::Down , false),
                        0b_0000_1_011 => *cell = Cell::SpecialTransmitting(Direction::Right, false),
                        0b_0000_1_100 => *cell = Cell::SpecialTransmitting(Direction::Up   , false),
                        0b_0000_1_101 => *cell = Cell::SpecialTransmitting(Direction::Left , false),
                        0b_0000_1_110 => *cell = Cell::SpecialTransmitting(Direction::Down , false),
                        0b_0000_1_111 => *cell = Cell::Confluence(false, false),
                        33.. => panic!("Каким-то образом было получено невозможное значение чувтвительной ячейки!"),
                        _ => {},
                    }

                },

                Cell::NormalTransmitting(_, v) |
                Cell::SpecialTransmitting(_, v)  => {

                    let event = events.iter().reduce(|r, e| {
                        match e {
                            Events::Break => e,

                            Events::Signal(true)
                            if *r != Events::Break => e,

                            Events::SignalDrift |
                            Events::Signal(_)
                            if *r == Events::SignalDrift || *r == Events::Signal(false) => r,

                            _ => panic!("В Transmitting ячейке не удалось обработать событие!"),
                        }
                    }).expect("В Transmitting ячейке не оказалось событий!");

                    match event {
                        Events::Signal(true) => *v = true,
                        Events::SignalDrift | Events::Signal(false) => *v = false,
                        Events::Break => *cell = Cell::Unexcitable,
                        _ => panic!("В Transmitting ячейке не удалось обработать событие!"),
                    }

                },

                Cell::Confluence(current, previous) => {

                    let event = events.iter().fold(Events::SignalDrift, |r, e| {
                        match e {
                            Events::Break => Events::Break,

                            Events::NotOut
                            if r != Events::Break => Events::NotOut,

                            Events::Signal(v)
                            if r != Events::Break || r != Events::NotOut =>
                                match r {
                                    Events::Signal(pv) => Events::Signal(*v && pv),
                                    _ =>  Events::Signal(*v),
                                }
                            ,

                            Events::SignalDrift => r,

                            _ => panic!("В Confluence ячейке не удалось обработать событие!"),
                        }
                    });

                    match event {
                        Events::Signal(v) => {
                            *previous = *current;
                            *current = v;
                        },

                        Events::SignalDrift => {
                            *previous = *current;
                            *current = false;
                        },

                        Events::Break => *cell = Cell::Unexcitable,

                        Events::NotOut => {
                            *previous = false;
                            *current = false;
                        },

                        _ => panic!("В Confluence ячейке не удалось обработать событие!"),
                    }

                },
            }
        }
    }
}

/// Энум, характеризующий ячейки.
/// Некоторые варианты содержат необходимую информацию.
#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Unexcitable,
    Sensetive(u8),
    NormalTransmitting(Direction, bool),
    SpecialTransmitting(Direction, bool),
    Confluence(bool, bool), // C_ab: a - текущее, b - предыдущее
}

// Здесь я задаю соответствие энума трейту Default, который позволяет выводить компилятору значение по умолчанию самостоятельно.
impl Default for Cell {
    fn default() -> Self {
        Self::Unexcitable
    }
}

/// Энум, в котором хранятся все возможные типы событий, которые могут происходить на поле между ячейками.
#[derive(PartialEq, Eq, Debug)]
pub enum Events {
    Signal(bool),
    SignalDrift,
    Break,
    NotOut,
    Tact,
}

/// Просто энум для удобоваримых названий направлений.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    /// Возвращает Some(true), если оси и направления двух элементов совпадают.
    fn is_same_direction(&self, other: &Self) -> Option<bool> {
        //
        // Маска работает так:
        //
        // Есть две пары битов начиная с младших разрядов
        // В каждой из пар младший разряд является значение, а старший буффером от переполнения
        //
        // Младшая пара обозначает направление по оси, а старшая - саму ось X или Y
        //
        // При сложении и и применении маски для сброса буферов в каждой паре остаётся по одному нужному нам биту
        // Этими битами являются младшие разряды в парах
        //
        // При значении 1 они означают, что оси или направления были разными
        // При значении 0 - что они были одинаковыми
        // 
        // Таким образом при минимальных усилиях мы можем определить относительные позиции.
        //
        let a: u8 = match self {
            Direction::Up => 0b_0000_01_00,
            Direction::Down => 0b_0000_01_01,
            Direction::Left => 0b_0000_00_00,
            Direction::Right => 0b_0000_00_01,
        };
        let b: u8 = match other {
            Direction::Up => 0b_0000_01_00,
            Direction::Down => 0b_0000_01_01,
            Direction::Left => 0b_0000_00_00,
            Direction::Right => 0b_0000_00_01,
        };
        
        match (a+b)^0b_0000_01_01 {
            0b_0000_00_00   => Some(true ),
            0b_0000_00_01   => Some(false),
            0b_0000_01_00 | 
            0b_0000_01_01   => None,
            _ => None, // Здесь я отделил от ветки выше из-за невозможности получения остальных значений.
        }
    }
}