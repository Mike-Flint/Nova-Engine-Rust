

use std::sync::Arc;

use cgmath::{Matrix4, SquareMatrix};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator, device::Queue, format::Format,
    image::view::ImageView, memory::allocator::StandardMemoryAllocator, sync::GpuFuture,
};

use crate::{
    graphics::frame::{NFrameSystem, Pass},
    graphics::systems::triangle::NTriangleDrawSystem,
};

#[derive(Clone)]
pub struct NAllocators {
    pub command_buffers: Arc<StandardCommandBufferAllocator>,  // Аллокатор командных буферов
    pub memory: Arc<StandardMemoryAllocator>,                  // Аллокатор памяти
}

// Основной пайплайн рендеринга
pub struct NRenderPipeline {
    frame_system: NFrameSystem,      // Система кадров
    draw_pipeline: NTriangleDrawSystem,  // Система отрисовки треугольников
}

impl NRenderPipeline {
    pub fn new(queue: Arc<Queue>, image_format: Format, allocators: &NAllocators) -> Self {
        let frame_system = NFrameSystem::new(queue.clone(), image_format, allocators.clone());
        let draw_pipeline =
            NTriangleDrawSystem::new(queue, frame_system.deferred_subpass(), allocators);

        Self { frame_system, draw_pipeline }
    }

    pub fn render(
        &mut self,
        before_future: Box<dyn GpuFuture>,  // Future от предыдущей операции
        image: Arc<ImageView>,              // Целевое изображение
    ) -> Box<dyn GpuFuture> {              // Возвращает Future завершения рендеринга
        let mut frame = self.frame_system.frame(
            before_future,
            image.clone(),
            Matrix4::identity(),
        );
        let dims = image.image().extent();
        // Draw each render pass that's related to scene
        let mut after_future = None;
        while let Some(pass) = frame.next_pass() {
            match pass {
                Pass::Deferred(mut draw_pass) => {
                    let cb = self.draw_pipeline.draw([dims[0], dims[1]]);
                    draw_pass.execute(cb);
                }
                Pass::Finished(af) => {
                    after_future = Some(af);
                }
            }
        }
        after_future.unwrap().then_signal_fence_and_flush().unwrap().boxed()
    }
}
