use std::sync::Arc;

use egui_dock::tree;
use egui::{Key, Ui};
use egui_winit::egui as egui;
use egui::{load::SizedTexture, Context, ImageSource, Visuals};
use egui_tiles::Tree;
use egui_tiles::Tile;

use egui_winit_vulkano::Gui;
use vulkano::{
    format::Format,
    image::view::ImageView,
};

use crate::ui::tiles::PaneTrait;
use crate::ui::tiles::{TileUI, show_tiles_ui};
use egui_winit::winit::event_loop::ActiveEventLoop;
use egui_winit_vulkano::{GuiConfig};
use crate::core::App;


// Структура GuiState управляет состоянием пользовательского интерфейса
pub struct GuiSystem {
    pub tile_ui: TileUI,
    pub gui: Gui
}

impl GuiSystem {
    // Створює GUI-ідентифікатори для зображень та налаштовує тайлову систему
    pub fn new( event_loop:& ActiveEventLoop , app: &mut App) -> GuiSystem {

        let gui = {
            let renderer = app.windows.get_primary_renderer_mut().unwrap();
            Gui::new(
                event_loop,
                renderer.surface(),
                renderer.graphics_queue(),
                renderer.swapchain_format(),
                GuiConfig::default(),
            )
        };

        let tile_ui = TileUI::new();

        GuiSystem {
            tile_ui,
            gui
        }
    }


    pub fn draw(&mut self) {

        let egui_context = {
            let gui = &mut self.gui;
            let mut ctx_opt = None;

            gui.immediate_ui(|gui| {
                ctx_opt = Some(gui.context().clone());
            });

            ctx_opt.unwrap()
        };

        egui_context.set_visuals(Visuals::dark());

        let mut pane_states: Vec<(String, bool)> = Vec::new();
        for (_tile_id, tile) in self.tile_ui.tree.tiles.iter() {
            if let Tile::Pane(pane) = tile {
                let base = pane.get_base(); // Використовуємо незмінне посилання
                pane_states.push((base.name.clone(), base.visible));
            }
        }

        egui::TopBottomPanel::top("top_bar").show(&egui_context, |ui| {
            egui::menu::bar(ui, |ui| {

                ui.menu_button("Windows", |ui| {

                    for (name, visible) in &mut pane_states {
                        ui.checkbox(visible, name.clone());
                    }
                });
            });
        });


        for (name, new_visibility) in pane_states {
            for (_tile_id, tile) in self.tile_ui.tree.tiles.iter_mut() {
                if let Tile::Pane(pane) = tile {
                    if pane.get_base().name == name {
                        // І оновлюємо її стан `visible`
                        pane.get_base_mut().visible = new_visibility;
                        break; // Переходимо до наступної панелі зі списку
                    }
                }
            }
        }

        update_tiles_visibility( &mut self.tile_ui, &egui_context);
        show_tiles_ui(&egui_context, &mut self.tile_ui);
    }
}

fn update_tiles_visibility( tile_ui: &mut TileUI, ui : &Context) {
    let pane_ids: Vec<egui_tiles::TileId> = tile_ui.tree.tiles
        .iter()
        .filter_map(|(tile_id, tile)| {
            if tile.is_pane() {
                Some(*tile_id)
            } else {
                None
            }
        })
        .collect();

    // Етап 2: Пройтися по ID і змінити ОРИГІНАЛЬНІ панелі
    for tile_id in pane_ids {
        // Отримуємо мутабельний доступ до тайлу в ОРИГІНАЛЬНОМУ дереві
        if let Some(Tile::Pane(pane)) = tile_ui.tree.tiles.get_mut(tile_id) {
            // Тепер `pane` - це `&mut Box<dyn PaneTrait>`, і ви можете його змінювати.
            // Приклад: перемикаємо видимість при натисканні на 'V'
            if ui.input(|i| i.key_pressed(egui::Key::V)) {
                let base = pane.get_base_mut();
                base.visible = !base.visible;
            }

            // Ваша логіка синхронізації (тепер вона працює з тим самим `tree`)
            let should_be_visible = pane.get_base().visible;
            let is_currently_visible = tile_ui.tree.is_visible(tile_id);

            if should_be_visible != is_currently_visible {
                tile_ui.tree.set_visible(tile_id, should_be_visible);
            }
        }
    }
}