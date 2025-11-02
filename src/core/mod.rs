mod context;
mod window;
mod time;

use std::sync::Arc;

use egui_winit::winit as winit;

use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::{
    command_buffer::allocator::{
        StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
};
use vulkano::swapchain::{PresentMode, SwapchainCreateInfo};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::DEFAULT_IMAGE_FORMAT,
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::{graphics::pipeline::NRenderPipeline,
            graphics::renderer::NRenderer,
            graphics::pipeline::NAllocators,
            ui::gui::GuiSystem,
            core::time::TimeInfo
        };

// Основная структура приложения
pub struct App {
    context: VulkanoContext,        // Контекст Vulkan
    pub windows: VulkanoWindows,        // Управление окнами
    scene_view_size: [u32; 2],     // Размер сцены
    pub scene_image: Arc<ImageView>,    // Изображение для рендеринга сцены
    time: TimeInfo,                 // Информация о времени и FPS
    renderer: NRenderer,  // Пайплайн рендеринга
    gui_system: Option<GuiSystem>,   // Состояние GUI
    is_minimized: bool,
}

impl Default for App {
    fn default() -> Self {
        // Vulkano context with explicit Vulkan configuration
        let mut config = VulkanoConfig::default();
        // config.device_features.shader_float64 = true; // Enable Vulkan features

        let context = VulkanoContext::new(config);

        // Vulkano windows
        let windows = VulkanoWindows::default();

        // Create renderer for our scene & ui
        let scene_view_size = [256, 256];
        // Create a simple image to which we'll draw the triangle scene
        let scene_image = ImageView::new_default(
            Image::new(
                context.memory_allocator().clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: DEFAULT_IMAGE_FORMAT,
                    extent: [scene_view_size[0], scene_view_size[1], 1],
                    array_layers: 1,
                    usage: ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let time = TimeInfo::new();

        // Create our render pipeline
        let renderer = NRenderer::new(&context);

        Self {
            context,
            windows,
            scene_view_size,
            scene_image,
            time,
            renderer,
            gui_system: None,
            is_minimized: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_descriptor = WindowDescriptor {
            title: "Nova-Engine".to_string(),
            // Вмикаємо повноекранний режим без рамок
            present_mode: PresentMode::Immediate,
            transparent: true,
            ..Default::default()
        };

        self.windows.create_window(event_loop, &self.context, &window_descriptor, |ci| {
            ci.image_format = Format::A2B10G10R10_UNORM_PACK32;
            ci.min_image_count = ci.min_image_count.max(2);
            ci.present_mode = vulkano::swapchain::PresentMode::Mailbox;
            
        });

        // Create gui state (pass anything your state requires)
        self.gui_system = Some(GuiSystem::new(event_loop, self));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let renderer = self.windows.get_renderer_mut(window_id).unwrap();

        

        if window_id == renderer.window().id() {
            {
                let gui: &mut Gui = &mut self.gui_system.as_mut().unwrap().gui;
                let _pass_events_to_game = !gui.update(&event);
            }
            match event {
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width == 0 || physical_size.height == 0 {
                        self.is_minimized = true;
                    } else {
                        self.is_minimized = false;
                        renderer.resize();
                    }
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    renderer.resize();
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if self.is_minimized {
                        return;
                    }

                    self.gui_system.as_mut().unwrap().draw();
                    // Render UI
                    // Acquire swapchain future
                    match renderer.acquire( None , |_| {}) {
                        Ok(future) => {
                            // Draw scene
                            let after_scene_draw =
                                self.renderer.render_pipeline.render(future, self.scene_image.clone());
                            // Render gui
                            let after_future = self.gui_system.as_mut().unwrap().gui
                                .draw_on_image(after_scene_draw, renderer.swapchain_image_view());
                            // Present swapchain
                            renderer.present(after_future, true);
                        }
                        Err(vulkano::VulkanError::OutOfDate) => {
                            renderer.resize();
                        }
                        Err(e) => panic!("Failed to acquire swapchain future: {}", e),
                    };
                    renderer.window().request_redraw();
                }
                _ => (),
            }
        }
    }
}

