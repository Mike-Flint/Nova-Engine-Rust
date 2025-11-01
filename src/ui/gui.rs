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
use crate::ui::tiles::Pane;


// Структура GuiState управляет состоянием пользовательского интерфейса
pub struct GuiState {
    // Флаги видимости окон
    show_texture_window1: bool,     // Показывать ли окно с текстурой дерева
    show_texture_window2: bool,     // Показывать ли окно с текстурой собаки
    show_scene_window: bool,        // Показывать ли окно со сценой
}

pub struct Guilayout {
    tile_ui: TileUI
}

impl Guilayout {
    // Створює GUI-ідентифікатори для зображень та налаштовує тайлову систему
    pub fn new(gui: &mut Gui, scene_image: Arc<ImageView>, scene_view_size: [u32; 2]) -> Guilayout {
        // Реєструємо текстури
        let image_texture_id1 = gui.register_user_image(
            include_bytes!("../assets/tree.png"),
            Format::R8G8B8A8_SRGB,
            Default::default(),
        );
        let image_texture_id2 = gui.register_user_image(
            include_bytes!("../assets/doge2.png"),
            Format::R8G8B8A8_SRGB,
            Default::default(),
        );

        let mut tile_ui = TileUI::new();
        // Register textures
        let scene_texture_id = gui.register_user_image_view(scene_image, Default::default());

        // tile_ui.add_image_tile("Tree".to_owned(), image_texture_id1, [256.0, 256.0]);


        Guilayout {
            tile_ui
        }
    }

    pub fn layout(&mut self, egui_context: Context, window_size: [f32; 2], fps: f32) {
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