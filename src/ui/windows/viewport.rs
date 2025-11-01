use egui_winit::egui::{Ui};

use crate::ui::tiles::*;

#[derive(Clone, Debug)] 
pub struct Viewport {
    pub base: BasePane,

}

impl Viewport{
    pub fn new(id: usize, name: String) -> Self {
        Viewport {
            base: BasePane {
                id,
                name,
                visible: true,
            },
        }
    }    
}

impl PaneTrait  for  Viewport {
    fn render(&mut self, ui: &mut Ui) {
        ui.label(format!("BasePane: {}", self.base.name));
    }

    fn get_base_mut(&mut self) -> &mut BasePane {
        &mut self.base
    }

    fn get_base(& self) -> & BasePane {
        & self.base
    }
}

impl CloneablePane for Viewport {
    fn clone_box(&self) -> Box<dyn PaneTrait> {
        Box::new(self.clone())
    }
}
