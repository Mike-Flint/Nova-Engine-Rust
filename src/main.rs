

#![allow(clippy::eq_op)]


pub mod ui;
pub mod scene;
mod graphics;
mod core;


use egui_winit::winit as winit;

use winit::{ event_loop::EventLoop};

use crate::core::App;


pub fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    // Event loop run
    event_loop.run_app(&mut app)
}




