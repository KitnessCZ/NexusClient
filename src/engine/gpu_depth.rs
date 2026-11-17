
/// GPU depth attachment used by the block renderer.
pub struct GpuDepthTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl GpuDepthTexture {
    /// Creates a depth texture for the given render dimensions.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("NexusClient Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
        }
    }

    /// Recreates the depth attachment after a render-size change.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) {
        let width = width.max(1);
        let height = height.max(1);

        if self.width == width && self.height == height {
            return;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("NexusClient Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.texture = texture;
        self.view = view;
        self.width = width;
        self.height = height;
    }

    /// Returns the depth texture view for a render pass.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the depth texture dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Keeps the underlying texture alive while this resource exists.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
