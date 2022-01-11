use crate::cellular_automaton::{CellularAutomation, Direction, State, State::*};
use crate::draw::CellularAutomatonPainter;
use crate::MouseButton;
use egui::{CtxRef, Ui};
use macroquad::input::{is_key_down, KeyCode};
use macroquad::prelude::{is_mouse_button_pressed, mouse_position, mouse_wheel};
use native_dialog::FileDialog;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::RwLock;

pub struct UI {
    cellular_automation: Rc<RwLock<CellularAutomation>>,
    painter: CellularAutomatonPainter,

    // Флаги
    f_redact_tact: bool,
    f_pause: bool,
    f_redact_field: bool,

    f_signal: bool,

    // Значения
    input_tact_text: String,
    input_tact: u64,
    speed: u64,
    selected_state: State,

    // История состояния полей (каждых 200 тактов)
    history: VecDeque<CellularAutomation>,
}

impl UI {
    pub fn new(cellular_automation: Rc<RwLock<CellularAutomation>>) -> Self {
        let input_tact = {
            let ca = cellular_automation.read().unwrap();
            ca.tact
        };

        let start = (*cellular_automation.read().unwrap().deref()).clone();

        Self {
            cellular_automation: cellular_automation.clone(),
            painter: CellularAutomatonPainter::new(cellular_automation.clone(), (50.0, 50.0)),

            f_redact_tact: false,
            f_pause: true,
            f_redact_field: false,

            f_signal: false,

            input_tact_text: input_tact.to_string(),
            input_tact,
            speed: 1,
            selected_state: Unexcitable,

            history: VecDeque::from([start]),
        }
    }

    pub fn draw(&mut self, ctx: &CtxRef) {
        // Обработка масштабирования

        if mouse_wheel() != (0.0, 0.0) {
            let (_, d) = mouse_wheel();
            let old_size = self.painter.cell_size.0;

            let new_size = if d > 0.0 {
                old_size + 5.0
            } else {
                old_size - 5.0
            };

            self.painter.change_cell_size((new_size, new_size));
        }

        // Обработка перемещения

        if is_key_down(KeyCode::Up) {
            self.painter.offset.1 += self.painter.cell_size.0;
        }

        if is_key_down(KeyCode::Left) {
            self.painter.offset.0 += self.painter.cell_size.0;
        }

        if is_key_down(KeyCode::Down) {
            self.painter.offset.1 -= self.painter.cell_size.0;
        }

        if is_key_down(KeyCode::Right) {
            self.painter.offset.0 -= self.painter.cell_size.0;
        }

        self.painter.draw();

        egui::SidePanel::right("GUI").show(ctx, |ui| {
            ui.heading("Клеточный автомат фон Неймана");
            ui.separator();

            if self.f_redact_field {
                self.draw_redact_field_page(ui);
            } else {
                self.draw_main_page(ui);
            }
        });
    }

    fn draw_main_page(&mut self, ui: &mut Ui) {
        ui.label("Для масштабирования использовать колёсико мыши или тачпад. \nДля перемещения поля зрения - стрелочки.");

        ui.collapsing("Дополнительная информация", |ui| {
            ui.label("Клеточный автомат с 29-ми состояниями.");
            ui.label("Поле бесконечно (насколько позволяет память компьютера) расширяется по мере необходимости.");
            ui.separator();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Такт: ");

            if self.f_redact_tact {
                ui.text_edit_singleline(&mut self.input_tact_text);

                let input = self.input_tact_text.parse::<u64>();

                match input {
                    Ok(n) => {
                        self.input_tact = n;
                    }

                    Err(_) => (),
                }
            } else {
                ui.label(self.input_tact.to_string());
            }

            if ui.button("R").clicked() {
                if self.f_redact_tact {
                    if self.input_tact > self.cellular_automation.read().unwrap().tact {
                        while self.input_tact > self.cellular_automation.write().unwrap().tact {
                            self.cellular_automation.write().unwrap().tact();
                        }
                    } else if self.input_tact < self.cellular_automation.read().unwrap().tact {
                        let mut dels = 0;

                        for (i, ca) in self.history.iter().enumerate() {
                            if ca.tact <= self.input_tact {
                                let c = Rc::new(RwLock::new(ca.clone()));

                                self.painter.cellular_automaton = c.clone();
                                self.cellular_automation = c;

                                dels = i;
                                break;
                            }
                        }

                        for _ in 0..dels {
                            self.history.pop_front();
                        }

                        while self.input_tact > self.cellular_automation.write().unwrap().tact {
                            self.cellular_automation.write().unwrap().tact();
                        }
                    }
                }

                self.f_redact_tact = !self.f_redact_tact;
                self.f_pause = true;

                self.input_tact_text = self.input_tact.to_string();
            }

            if !self.f_redact_tact && ui.button("Сделать шаг").clicked() {
                self.input_tact += 1;
                self.cellular_automation.write().unwrap().tact();
            }
        });

        ui.horizontal(|ui| {
            if ui.button(if self.f_pause { "P" } else { "G" }).clicked() {
                self.f_pause = !self.f_pause;
            }

            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Скорость: ");

                    if ui.button("-").clicked() {
                        self.speed -= 1;
                    }

                    ui.label(self.speed.to_string());

                    if ui.button("+").clicked() {
                        self.speed += 1;
                    }
                });
            });
        });

        ui.collapsing(
            "Дополнительная информация про кнопки",
            |ui| {
                ui.label("При нажатии на кнопку \"R\" можно перемотать к какому-либо такту.");
                ui.label("При нажатии на кнопку \"Сделать шаг\" можно сделать ровно один шаг.");

                if !self.f_redact_tact {
                    if self.f_pause {
                        ui.label("Нажмите на кнопку \"P\", чтобы снять поле с паузы.");
                    } else {
                        ui.label("Нажмите на кнопку \"G\", чтобы поставить поле на паузу.");
                    }

                    ui.label(
                        "Скорость изменения тактов можно изменить с помощью кнопок \"+\" и \"-\"",
                    );
                }
            },
        );

        ui.separator();

        if ui.button("Редактировать поле").clicked() {
            self.f_redact_field = true;
        }

        ui.separator();

        if ui.button("Загрузить поле").clicked() {
            let path = FileDialog::new()
                .set_location("~/")
                .add_filter("data", &["data"])
                .show_open_single_file()
                .unwrap();

            match path {
                None => {}
                Some(path) => {
                    let mut file = File::open(path.to_str().unwrap()).unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    let deserialized: CellularAutomation = serde_json::from_str(&contents).unwrap();

                    self.history = VecDeque::from([deserialized.clone()]);

                    let tact = deserialized.tact;
                    let ca = Rc::new(RwLock::new(deserialized));

                    self.cellular_automation = ca.clone();
                    self.painter = CellularAutomatonPainter::new(ca.clone(), (50.0, 50.0));
                    self.input_tact = tact;
                    self.input_tact_text = tact.to_string();
                }
            };
        }

        if ui.button("Выгрузить поле").clicked() {
            let path = FileDialog::new()
                .set_location("~/")
                .add_filter("data", &["data"])
                .show_save_single_file()
                .unwrap();

            match path {
                None => {}
                Some(path) => {
                    let serialized =
                        serde_json::to_string(&(*self.cellular_automation.read().unwrap().deref()))
                            .unwrap();

                    let mut file = File::create(path.to_str().unwrap()).unwrap();

                    file.write_all(serialized.as_bytes()).unwrap();
                }
            };
        }

        if !self.f_pause {
            let mut ca = self.cellular_automation.write().unwrap();

            self.input_tact += self.speed;

            for _ in 0..self.speed {
                ca.tact();

                if ca.tact % 200 == 0 {
                    self.history.push_front(ca.clone());
                }
            }
        }
    }

    fn draw_redact_field_page(&mut self, ui: &mut Ui) {
        // Обработка нажатия и изменения клетки
        if is_mouse_button_pressed(MouseButton::Right) {
            {
                let (x, y) = mouse_position(); // Получение сырой позиции

                // Применение смещения
                let (x, y) = (x - self.painter.offset.0, y - self.painter.offset.1);

                // Приведение к координатам ячеек
                let (x, y) = (
                    (x / self.painter.cell_size.0).ceil() as i64 - 1,
                    (y / self.painter.cell_size.1).ceil() as i64 - 1,
                );

                let mut ca = self.cellular_automation.write().unwrap();

                ca.set_cell((x, y), self.selected_state);
            }

            self.history
                .push_front((*self.cellular_automation.read().unwrap().deref()).clone());
        }

        ui.label("Для изменения ячейки кликните по ней правой кнопкой мыши");
        ui.separator();

        ui.checkbox(
            &mut self.f_signal,
            "Сигнал (только для транспортных клеток)",
        );

        ui.label("Состояния: ");

        ui.horizontal(|ui| {
            ui.radio_value(&mut self.selected_state, Unexcitable, Unexcitable);
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                Sensitive { value: 1u8 },
                Sensitive { value: 1u8 },
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                NormalTransmitting {
                    dir: Direction::Up,
                    signal: self.f_signal,
                },
                "Обычная транспортная ячейка вверх",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                NormalTransmitting {
                    dir: Direction::Left,
                    signal: self.f_signal,
                },
                "Обычная транспортная ячейка влево",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                NormalTransmitting {
                    dir: Direction::Down,
                    signal: self.f_signal,
                },
                "Обычная транспортная ячейка вниз",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                NormalTransmitting {
                    dir: Direction::Right,
                    signal: self.f_signal,
                },
                "Обычная транспортная ячейка вправо",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                SpecialTransmitting {
                    dir: Direction::Up,
                    signal: self.f_signal,
                },
                "Специальная транспортная ячейка вверх",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                SpecialTransmitting {
                    dir: Direction::Left,
                    signal: self.f_signal,
                },
                "Специальная транспортная ячейка влево",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                SpecialTransmitting {
                    dir: Direction::Down,
                    signal: self.f_signal,
                },
                "Специальная транспортная ячейка вниз",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                SpecialTransmitting {
                    dir: Direction::Right,
                    signal: self.f_signal,
                },
                "Специальная транспортная ячейка вправо",
            );
        });

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.selected_state,
                Confluence {
                    old: false,
                    new: false,
                },
                Confluence {
                    old: false,
                    new: false,
                },
            );
        });

        ui.separator();

        if ui.button("Назад").clicked() {
            self.f_redact_field = false;
        }
    }
}
