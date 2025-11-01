// Імпортуємо необхідні компоненти з бібліотек
use egui_dock::{DockState, TabViewer};  // DockState керує розташуванням вкладок, TabViewer відповідає за їх відображення
use egui_winit::egui::{self, Context, ImageSource};  // Компоненти для UI
use egui::load::SizedTexture;  // Для роботи з текстурами в UI

/// Структура, яка відповідає за відображення вмісту вкладок
/// Реалізує логіку того, як саме показувати різні типи контенту
pub struct NovaTabViewer {
    tabs: Vec<Tab>  // Зберігаємо список вкладок (наразі не використовується)
}

impl NovaTabViewer {
    /// Створює новий екземпляр viewer-а
    fn new() -> Self {
        Self { tabs: Vec::new() }
    }
}

/// Реалізація трейту TabViewer - основний інтерфейс для відображення вкладок
impl TabViewer for NovaTabViewer {
    type Tab = Tab;  // Вказуємо, який тип використовується для вкладок

    /// Відповідає за відображення вмісту вкладки
    /// ui: інтерфейс для малювання елементів
    /// tab: вкладка, яку потрібно відобразити
    fn ui(&mut self, ui: &mut egui_winit::egui::Ui, tab: &mut Self::Tab) {
        // Залежно від типу контенту вкладки, відображаємо різні елементи
        match &tab.content {
            TabContent::Empty => {
                ui.label("Empty tab");  // Порожня вкладка
            }
            TabContent::Scene { texture_id, size } => {
                // Відображаємо сцену як текстуру з вказаним розміром
                ui.image(ImageSource::Texture(SizedTexture::new(
                    *texture_id,
                    [size[0] as f32, size[1] as f32],
                )));
            }
            TabContent::Image { texture_id, size } => {
                // Відображаємо звичайне зображення
                ui.image(ImageSource::Texture(SizedTexture::new(*texture_id, *size)));
            }
            TabContent::Text(text) => {
                // Відображаємо текстовий вміст
                ui.label(text);
            }
        }
    }

    /// Повертає заголовок вкладки, який буде показано в її табі
    fn title(&mut self, tab: &mut Self::Tab) -> egui_winit::egui::WidgetText {
        (&tab.name).into()  // Використовуємо поле name як заголовок
    }
}

/// Перелічення для різних типів вмісту вкладок
#[derive(Default)]
pub enum TabContent {
    #[default]
    Empty,              // Порожня вкладка
    Scene {            // Вкладка для відображення сцени
        texture_id: egui::TextureId,  // ID текстури сцени
        size: [u32; 2],               // Розмір в пікселях [ширина, висота]
    },
    Image {            // Вкладка для відображення зображення
        texture_id: egui::TextureId,  // ID текстури зображення
        size: [f32; 2],               // Розмір для відображення [ширина, висота]
    },
    Text(String),      // Вкладка з текстовим вмістом
}

/// Структура, що представляє одну вкладку
pub struct Tab {
    pub name: String,      // Назва вкладки
    pub content: TabContent,  // Вміст вкладки
}

/// Головна структура для управління док-системою
pub struct DockUI {
    pub dock_state: DockState<Tab>,  // Стан док-системи (розташування вкладок)
    pub viewer: NovaTabViewer,             // Компонент для відображення вкладок
}

impl DockUI {
    /// Створює нову док-систему з початковими вкладками
    pub fn new() -> Self {
        // Створюємо порожній стан док-системи
        let dock_state = DockState::new(vec![]);


        // Повертаємо налаштовану док-систему
        Self { 
            dock_state,
            viewer: NovaTabViewer::new(),
        }
    }

    pub fn add_scene_tab(&mut self, texture_id: egui::TextureId, size: [u32; 2]) {
        self.dock_state.push_to_focused_leaf(Tab {
            name: "Scene".to_owned(),
            content: TabContent::Scene { texture_id, size },
        });
    }

    pub fn add_image_tab(&mut self, name: String, texture_id: egui::TextureId, size: [f32; 2]) {
        self.dock_state.push_to_focused_leaf(Tab {
            name,
            content: TabContent::Image { texture_id, size },
        });
    }
}


pub fn show_dock_ui(dock_ui: &mut DockUI, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui_dock::DockArea::new(&mut dock_ui.dock_state)
            .show_inside(ui, &mut dock_ui.viewer);
    });


}

impl TabViewer for DockUI {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match &tab.content {
            TabContent::Empty => {
                ui.label("Empty tab");
            }
            TabContent::Scene { texture_id, size } => {
                ui.image(ImageSource::Texture(SizedTexture::new(
                    *texture_id,
                    [size[0] as f32, size[1] as f32],
                )));
            }
            TabContent::Image { texture_id, size } => {
                ui.image(ImageSource::Texture(SizedTexture::new(*texture_id, *size)));
            }
            TabContent::Text(text) => {
                ui.label(text);
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&tab.name).into()
    }
}

