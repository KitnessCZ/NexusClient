
use super::{convert_vertices, vertices_as_bytes, ChunkMesh};

/// GPU-side representation of one chunk mesh.
///
/// Vertex and index data are uploaded once when this resource is created.
/// Recreate it when the corresponding CPU chunk mesh changes.
pub struct GpuChunkMesh {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
}

impl GpuChunkMesh {
    /// Uploads a CPU chunk mesh into GPU buffers.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &ChunkMesh,
    ) -> Self {
        if mesh.is_empty() || mesh.vertices.is_empty() {
            return Self {
                vertex_buffer: None,
                index_buffer: None,
                index_count: 0,
            };
        }

        let vertices = convert_vertices(&mesh.vertices);
        let vertex_bytes = vertices_as_bytes(&vertices);
        let index_bytes = bytemuck::cast_slice(&mesh.indices);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NexusClient Chunk Vertex Buffer"),
            size: vertex_bytes.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&vertex_buffer, 0, vertex_bytes);

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NexusClient Chunk Index Buffer"),
            size: index_bytes.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&index_buffer, 0, index_bytes);

        Self {
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            index_count: mesh.indices.len().min(u32::MAX as usize) as u32,
        }
    }

    /// Returns true when this mesh contains drawable geometry.
    pub fn is_empty(&self) -> bool {
        self.index_count == 0
            || self.vertex_buffer.is_none()
            || self.index_buffer.is_none()
    }

    /// Returns the number of indices uploaded to the GPU.
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    /// Encodes this chunk mesh into an active render pass.
    ///
    /// The caller must set the render pipeline and bind groups first.
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.is_empty() {
            return;
        }

        let Some(vertex_buffer) = self.vertex_buffer.as_ref() else {
            return;
        };

        let Some(index_buffer) = self.index_buffer.as_ref() else {
            return;
        };

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
