//! Helpers for renderer passes

use wgpu::StoreOp;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::window::WindowBuffers;

/// Create an attachment for the depth buffer that doesn't clear it.
pub fn create_default_depth_stencil_attachment(
    depth_buffer: &wgpu::TextureView,
) -> wgpu::RenderPassDepthStencilAttachment {
    wgpu::RenderPassDepthStencilAttachment {
        view: &(depth_buffer),
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: StoreOp::Store
        }),
        stencil_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: StoreOp::Store
        })
    }
}

/// Create a render pass that renders to the multisampled frame buffer without resolving and without clearing.
pub fn create_default_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    buffers: WindowBuffers<'a>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Option::from(wgpu::RenderPassColorAttachment {
            view: &(buffers.multisampled_texture_buffer),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: StoreOp::Store
            },
        })],
        depth_stencil_attachment: Some(create_default_depth_stencil_attachment(
            buffers.depth_buffer,
        )),
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}

/// Encode a render pass to resolve the multisampled frame buffer to the window frame buffer
pub fn encode_resolve_render_pass<'a>(encoder: &mut wgpu::CommandEncoder, buffers: WindowBuffers) {
    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Option::from(wgpu::RenderPassColorAttachment {
            view: &(buffers.multisampled_texture_buffer),
            resolve_target: Some(buffers.texture_buffer),
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: StoreOp::Store
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}

pub fn create_clear_color_attachment(
    buffers: WindowBuffers,
) -> [wgpu::RenderPassColorAttachment; 1] {
    [wgpu::RenderPassColorAttachment {
        view: &(buffers.multisampled_texture_buffer),
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(crate::window::CLEAR_COLOR),
            store: StoreOp::Store
        },
    }]
}

pub fn create_clear_depth_attachment(
    buffers: WindowBuffers,
) -> wgpu::RenderPassDepthStencilAttachment {
    wgpu::RenderPassDepthStencilAttachment {
        view: &(buffers.depth_buffer),
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(crate::window::CLEAR_DEPTH),
            store: StoreOp::Store
        }),
        stencil_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: StoreOp::Store
        })
    }
}

/// Clear the multisampled color buffer and the depth buffer
pub fn clear_color_and_depth(encoder: &mut wgpu::CommandEncoder, buffers: WindowBuffers) {
    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}
/// Clear the depth buffer
pub fn clear_depth(encoder: &mut wgpu::CommandEncoder, buffers: WindowBuffers) {
    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[],
        depth_stencil_attachment: Some(create_clear_depth_attachment(buffers)),
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}

/// Convert a vector to a buffer compatible slice of u8
pub fn to_u8_slice<T: Copy>(v: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * std::mem::size_of::<T>()) }
}

/// Helper to create a buffer from an existing slice.
pub fn buffer_from_slice(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8]) -> wgpu::Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        usage,
        contents: &data
    })
}

