use egui_dock::egui::Button;
use egui_winit::egui::{self, Context, ImageSource, Ui, WidgetText, Vec2};
use egui_winit::egui::load::SizedTexture;
use egui_tiles::{TileId, Tree, Tiles};
use std::cell::RefCell;
use std::sync::Arc;
use egui::{CursorIcon};

use crate::ui::windows::{content_browser::ContentBrowser,
                            details::Details, 
                            hierarchy::Hierarchy, 
                            viewport::Viewport};

#[derive(Debug, Clone)]
pub struct BasePane {
    pub id: usize,
    pub name: String,
    pub visible: bool,
}


pub trait CloneablePane {
    fn clone_box(&self) -> Box<dyn PaneTrait>;
}

pub trait PaneTrait: CloneablePane {
    fn render(&mut self, ui: &mut Ui);

    fn get_base_mut(&mut self) -> &mut BasePane;
    fn get_base(& self) -> & BasePane;

    

    fn base_settings(&mut self, ui: &mut Ui) -> egui_tiles::UiResponse {
        let base = self.get_base_mut();

        let button_rect = ui.horizontal(|ui| {
            ui.label(base.name.as_str());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").on_hover_text("Close").clicked() {
                    base.visible = false;
                }
            });
        }).response.rect;

        let drag_height = 15.0;
        let rect = egui::Rect::from_min_size(
            ui.min_rect().min,                   // верх окна
            egui::vec2(ui.available_width() - button_rect.height() - 2f32, drag_height),
        );

        // регистрируем область для взаимодействия
        let response = ui.interact(
            rect,
            ui.make_persistent_id("drag_area"), // уникальный ID
            egui::Sense::drag(),
        ).on_hover_cursor(CursorIcon::Move);

        // если клик/перетаскивание началось
        if response.drag_started() {
            return egui_tiles::UiResponse::DragStarted;
        }

        // --- остальная часть окна ---
        egui_tiles::UiResponse::None
    }
}

impl Clone for Box<dyn PaneTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub type Pane = Box<dyn PaneTrait>;


/// Main structure for tile system management


#[derive(Clone)]
pub struct TileUI {
    pub tree: Tree<Pane>,
    next_pane_nr: usize,
}

impl TileUI {
    pub fn new() -> Self {
        let mut tiles: Tiles<Pane> = Tiles::default();
        let mut next_pane_nr = 0;

        // Create initial tiles
        let mut tabs = vec![];

        // Create a horizontal layout of 3 panes
        let horizontal_panes = Box::new(ContentBrowser::new(next_pane_nr, "Content Browser".into()));
        next_pane_nr += 1;
        tabs.push(tiles.insert_pane(horizontal_panes));

        let horizontal_panes = Box::new(Details::new(next_pane_nr, "Details".into()));
        next_pane_nr += 1;
        tabs.push(tiles.insert_pane(horizontal_panes));

        // Add a single pane as a tab
        let horizontal_panes = Box::new(Viewport::new(next_pane_nr, "Viewport".into()));
        next_pane_nr += 1;
        tabs.push(tiles.insert_pane(horizontal_panes));

        let horizontal_panes = Box::new(Hierarchy::new(next_pane_nr, "Hierarchy".into()));
        next_pane_nr += 1;
        tabs.push(tiles.insert_pane(horizontal_panes));

        // Create the root tab tile
        let root = tiles.insert_horizontal_tile(tabs);
        let tree: Tree<Pane> = Tree::new("my_tree", root, tiles.clone());

        Self {tree, next_pane_nr}
    }
}


pub fn show_tiles_ui( ctx: &Context, tile_ui : &mut TileUI) {
    egui::CentralPanel::default().show(ctx, |ui| {
        tile_ui.tree.ui(&mut tile_ui.clone(), ui);
    });
}

impl egui_tiles::Behavior<Pane> for TileUI {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.get_base().name.as_str().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        // Give each pane a unique color:
        let color = egui::epaint::Color32::from_rgb(27, 27, 27);
        ui.painter().rect_filled(ui.max_rect(), 1.0, color);


        let response = pane.base_settings( ui);
        
        pane.render(ui);

        response
    }
}



