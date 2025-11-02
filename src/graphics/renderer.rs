use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::DEFAULT_IMAGE_FORMAT,
    window::{VulkanoWindows, WindowDescriptor},
};
use std::sync::Arc;

use vulkano::{
    command_buffer::allocator::{
        StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
    }};


use crate::graphics::{pipeline::NRenderPipeline,
                        pipeline::NAllocators
};


pub struct NRenderer{
    pub render_pipeline: NRenderPipeline,
}

impl NRenderer {
    pub fn new(context: &VulkanoContext) -> Self {
        Self{
            render_pipeline: NRenderPipeline::new(
                context.graphics_queue().clone(),
                DEFAULT_IMAGE_FORMAT,
                &NAllocators {
                    command_buffers: Arc::new(StandardCommandBufferAllocator::new(
                        context.device().clone(),
                        StandardCommandBufferAllocatorCreateInfo {
                            secondary_buffer_count: 32,
                            ..Default::default()
                        },
                    )),
                    memory: context.memory_allocator().clone(),
                },
            )
        }
    }
}