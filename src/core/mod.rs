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
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::DEFAULT_IMAGE_FORMAT,
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::{graphics::renderer::RenderPipeline,
            graphics::renderer,
            ui::gui::Guilayout,
            core::time::TimeInfo
        };

// Основная структура приложения
pub struct App {
    context: VulkanoContext,        // Контекст Vulkan
    windows: VulkanoWindows,        // Управление окнами
    scene_view_size: [u32; 2],     // Размер сцены
    scene_image: Arc<ImageView>,    // Изображение для рендеринга сцены
    time: TimeInfo,                 // Информация о времени и FPS
    scene_render_pipeline: RenderPipeline,  // Пайплайн рендеринга
    gui: Option<Gui>,              // GUI система
    gui_state: Option<Guilayout>,   // Состояние GUI
}

impl Default for App {
    fn default() -> Self {
        // Vulkano context
        let context = VulkanoContext::new(VulkanoConfig::default());

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
        let scene_render_pipeline = RenderPipeline::new(
            context.graphics_queue().clone(),
            DEFAULT_IMAGE_FORMAT,
            &renderer::Allocators {
                command_buffers: Arc::new(StandardCommandBufferAllocator::new(
                    context.device().clone(),
                    StandardCommandBufferAllocatorCreateInfo {
                        secondary_buffer_count: 32,
                        ..Default::default()
                    },
                )),
                memory: context.memory_allocator().clone(),
            },
        );

        Self {
            context,
            windows,
            scene_view_size,
            scene_image,
            time,
            scene_render_pipeline,
            gui: None,
            gui_state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Коли застосунок "продовжується" — створюється вікно і GUI
        // 1) create_window -> створює renderer для цього вікна
        // 2) Gui::new створює egui обгортку, прив'язану до renderer-а (swapchain, queue, формат)
        // 3) GuiState::new реєструє текстури та view на scene_image, щоб UI міг їх показувати
        self.windows.create_window(event_loop, &self.context, &WindowDescriptor::default(), |ci| {
            ci.image_format = Format::B8G8R8A8_UNORM;
            ci.min_image_count = ci.min_image_count.max(2);
            ci.present_mode = vulkano::swapchain::PresentMode::Mailbox;
            
        });
        // Create gui as main render pass (no overlay means it clears the image each frame)
        let mut gui = {
            let renderer = self.windows.get_primary_renderer_mut().unwrap();
            Gui::new(
                event_loop,
                renderer.surface(),
                renderer.graphics_queue(),
                renderer.swapchain_format(),
                GuiConfig::default(),
            )
        };

        // Create gui state (pass anything your state requires)
        self.gui_state =
            Some(Guilayout::new(&mut gui, self.scene_image.clone(), self.scene_view_size));

        self.gui = Some(gui);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Основний обробник подій від winit:
        // - якщо це наше головне вікно, відправляємо подію в egui (gui.update).
        // - при RedrawRequested:
        //     a) виконуємо immediate UI (викликаємо GuiState::layout),
        //     b) acquire swapchain image,
        //     c) рендеримо сцену в scene_image (RenderPipeline::render),
        //     d) рендеримо GUI поверх swapchain image (gui.draw_on_image),
        //     e) present.
        // - при Resize/ScaleFactorChanged: викликаємо renderer.resize(), щоб оновити swapchain.
        // - при CloseRequested: виходимо з event loop.
        let renderer = self.windows.get_renderer_mut(window_id).unwrap();

        let gui = self.gui.as_mut().unwrap();

        if window_id == renderer.window().id() {
            let _pass_events_to_game = !gui.update(&event);
            match event {
                WindowEvent::Resized(_) => {
                    renderer.resize();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    renderer.resize();
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                _ => (),
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // Викликається коли цикл подій планує чекати — тут запрошуємо redraw,
        // щоб відмалювати наступний кадр. Це забезпечує постійне оновлення UI/сцени.

        let renderer = self.windows.get_primary_renderer_mut().unwrap();
        let gui = self.gui.as_mut().unwrap();

        gui.immediate_ui(|gui| {
                        let ctx = gui.context();
                        self.gui_state.as_mut().unwrap().layout(
                            ctx,
                            renderer.window_size(),
                            400f32,
                        )
                    });
                    // Render UI
                    // Acquire swapchain future
                    match renderer.acquire(Some(std::time::Duration::from_millis(0)), |_| {}) {
                        Ok(future) => {
                            // Draw scene
                            let after_scene_draw =
                                self.scene_render_pipeline.render(future, self.scene_image.clone());
                            // Render gui
                            let after_future = gui
                                .draw_on_image(after_scene_draw, renderer.swapchain_image_view());
                            // Present swapchain
                            renderer.present(after_future, true);
                        }
                        Err(vulkano::VulkanError::OutOfDate) => {
                            renderer.resize();
                        }
                        Err(e) => panic!("Failed to acquire swapchain future: {}", e),
                    };



        _event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }
}

