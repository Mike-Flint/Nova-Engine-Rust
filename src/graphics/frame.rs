// Copyright (c) 2017 The vulkano developers <=== !
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

// This is a simplified version of the example. See that for commented version of this code.
// https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/deferred/frame/system.rs
// Egui drawing could be its own pass or it could be a deferred subpass

use std::sync::Arc;

use cgmath::Matrix4;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents,
    },
    device::Queue,
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};

use crate::graphics::renderer::Allocators;

/// Система для рендеринга одного кадра
pub struct FrameSystem {
    gfx_queue: Arc<Queue>,          // Очередь графических команд
    render_pass: Arc<RenderPass>,   // Проход рендеринга
    depth_buffer: Arc<ImageView>,   // Буфер глубины
    allocators: Allocators,         // Аллокаторы памяти и команд
}

impl FrameSystem {
    pub fn new(
        gfx_queue: Arc<Queue>,
        final_output_format: Format,
        allocators: Allocators,
    ) -> FrameSystem {
        let render_pass = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    format: final_output_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            passes: [
                {
                    color: [final_color],
                    depth_stencil: {depth},
                    input: []
                }
            ]
        )
        .unwrap();

        let depth_buffer = Image::new(
            allocators.memory.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: [1, 1, 1],
                array_layers: 1,
                usage: ImageUsage::SAMPLED | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();
        let depth_buffer = ImageView::new_default(depth_buffer.clone()).unwrap();
        FrameSystem { gfx_queue, render_pass, depth_buffer, allocators }
    }

    #[inline]
    pub fn deferred_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    pub fn frame<F>(
        &mut self,
        before_future: F,
        final_image: Arc<ImageView>,
        world_to_framebuffer: Matrix4<f32>,
    ) -> Frame <'_>
    where
        F: GpuFuture + 'static,
    {
        // Пояснення:
        // - Оновлюємо depth_buffer під розміри final_image (якщо потрібно).
        // - Створюємо Framebuffer з final_image + depth_buffer.
        // - Починаємо primary AutoCommandBufferBuilder з begin_render_pass(..., SubpassContents::SecondaryCommandBuffers),
        //   що дозволяє виконувати всередині вторинні командні буфери (згенеровані draw системою).
        // - Повертаємо Frame, яка дає next_pass() для послідовного виконання пасів.

        let img_dims = final_image.image().extent();
        if self.depth_buffer.image().extent() != img_dims {
            self.depth_buffer = ImageView::new_default(
                Image::new(
                    self.allocators.memory.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::D16_UNORM,
                        extent: final_image.image().extent(),
                        array_layers: 1,
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT
                            | ImageUsage::TRANSIENT_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap();
        }
        let framebuffer = Framebuffer::new(self.render_pass.clone(), FramebufferCreateInfo {
            attachments: vec![final_image, self.depth_buffer.clone()],
            ..Default::default()
        })
        .unwrap();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.allocators.command_buffers.clone(),
            self.gfx_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                },
            )
            .unwrap();

        Frame {
            system: self,
            num_pass: 0,
            before_main_cb_future: Some(Box::new(before_future)),
            framebuffer,
            recording_command_buffer: Some(command_buffer_builder),
            world_to_framebuffer,
        }
    }
}

// Frame/Pass/DrawPass пояснення:
// - Frame::next_pass повертає або DrawPass (для малювання сцени) або Finished (коли все побудовано).
// - DrawPass::execute отримує SecondaryAutoCommandBuffer і вставляє його в primary builder.
// - Коли всі паси завершені, primary command buffer будується та виконуються залежності (before_future -> primary CB).
pub struct Frame<'a> {
    system: &'a mut FrameSystem,    // Ссылка на систему кадров
    num_pass: u8,                   // Номер текущего прохода
    before_main_cb_future: Option<Box<dyn GpuFuture>>,  // Future для синхронизации
    framebuffer: Arc<Framebuffer>,  // Фреймбуфер для рендеринга
    recording_command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>, // Строитель команд
    world_to_framebuffer: Matrix4<f32>, // Матрица преобразования мировых координат
}

impl<'a> Frame<'a> {
    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 'a>> {
        let current_pass = {
            let current_pass = self.num_pass;
            self.num_pass += 1;
            current_pass
        };
        match current_pass {
            0 => Some(Pass::Deferred(DrawPass { frame: self })),
            1 => {
                self.recording_command_buffer
                    .as_mut()
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();
                let command_buffer = self.recording_command_buffer.take().unwrap().build().unwrap();
                let after_main_cb = self
                    .before_main_cb_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.gfx_queue.clone(), command_buffer)
                    .unwrap();
                Some(Pass::Finished(Box::new(after_main_cb)))
            }
            _ => None,
        }
    }
}

pub enum Pass<'f, 's: 'f> {
    Deferred(DrawPass<'f, 's>),
    Finished(Box<dyn GpuFuture>),
}

pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    #[inline]
    pub fn execute(&mut self, command_buffer: Arc<SecondaryAutoCommandBuffer>) {
        self.frame
            .recording_command_buffer
            .as_mut()
            .unwrap()
            .execute_commands(command_buffer)
            .unwrap();
    }

    #[allow(dead_code)]
    #[inline]
    pub fn viewport_dimensions(&self) -> [u32; 2] {
        self.frame.framebuffer.extent()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn world_to_framebuffer_matrix(&self) -> Matrix4<f32> {
        self.frame.world_to_framebuffer
    }
}

// FrameSystem: обгортка над render pass-ом та буферами, яка відповідає за підготовку кадру.
// Ключові кроки:
// - створює RenderPass з color + depth attachments,
// - містить depth_buffer який підлаштовується під розмір фінального зображення,
// - frame(...) повертає структуру Frame яка дозволяє виконати послідовність пасів:
//     * Deferred: місце для малювання сцени (Secondary command buffers),
//     * Finished: коли завершено запис, будується primary command buffer і виконується на черзі.
