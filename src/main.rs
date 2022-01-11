use macroquad::prelude::*;
use std::rc::Rc;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod cellular_automaton;
mod draw;
mod ui;

use crate::cellular_automaton::CellularAutomation;
use crate::ui::UI;

#[macroquad::main("Клеточный автомат фон Неймана (наверное работает)")]
async fn main() {
    let ca = CellularAutomation::new();

    let mut ui = UI::new(Rc::new(RwLock::new(ca)));

    loop {
        let now = Instant::now();

        clear_background(WHITE);

        egui_macroquad::ui(|ctx| {
            ui.draw(ctx);
        });

        egui_macroquad::draw();

        // Ожидание примерного времени для держания FPS в районе 60
        let elapsed_time = now.elapsed().as_millis();

        if elapsed_time < 16 {
            sleep(Duration::from_millis((16 - elapsed_time) as u64));
        }

        next_frame().await;
    }
}
