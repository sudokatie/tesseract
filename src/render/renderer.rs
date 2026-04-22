//! Main renderer responsible for orchestrating render passes.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor,
    BufferUsages, Color, CommandEncoderDescriptor, Device, Extent3d, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    SamplerDescriptor, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

use super::pipeline::{
    camera_bind_group_layout, light_bind_group_layout, material_bind_group_layout, RenderPipeline,
};
use super::shadows::ShadowMapper;
use super::{Camera, Light, LightKind, Mesh, PbrMaterial};

/// Camera uniform for GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub position: [f32; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera, transform_pos: Vec3, aspect: f32) -> Self {
        let view = Mat4::look_at_rh(transform_pos, transform_pos + Vec3::NEG_Z, Vec3::Y);
        let projection = camera.projection_matrix(aspect);
        let view_projection = projection * view;

        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            view_projection: view_projection.to_cols_array_2d(),
            position: [transform_pos.x, transform_pos.y, transform_pos.z, 1.0],
        }
    }
}

/// Light uniform for GPU (single light).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightUniform {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4], // intensity, range, inner_cone, outer_cone
    pub light_type: u32,
    pub _padding: [u32; 3],
}

impl LightUniform {
    pub fn from_light(light: &Light, position: Vec3, direction: Vec3) -> Self {
        let light_type = match light.kind {
            LightKind::Directional => 0,
            LightKind::Point { .. } => 1,
            LightKind::Spot { .. } => 2,
            LightKind::Ambient => 3,
        };

        let (range, inner_cone, outer_cone) = match light.kind {
            LightKind::Point { range } => (range, 0.0, 0.0),
            LightKind::Spot { angle, range } => {
                // Use angle for both inner and outer (simple falloff)
                let inner = (angle * 0.8).cos();
                let outer = angle.cos();
                (range, inner, outer)
            }
            _ => (0.0, 0.0, 0.0),
        };

        Self {
            position: [position.x, position.y, position.z, 1.0],
            direction: [direction.x, direction.y, direction.z, 0.0],
            color: [light.color.x, light.color.y, light.color.z, 1.0],
            params: [light.intensity, range, inner_cone, outer_cone],
            light_type,
            _padding: [0; 3],
        }
    }
}

/// Lights uniform buffer (up to 16 lights).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightsUniform {
    pub lights: [LightUniform; 16],
    pub light_count: u32,
    pub _padding: [u32; 3],
}

impl Default for LightsUniform {
    fn default() -> Self {
        Self {
            lights: [LightUniform {
                position: [0.0; 4],
                direction: [0.0; 4],
                color: [0.0; 4],
                params: [0.0; 4],
                light_type: 0,
                _padding: [0; 3],
            }; 16],
            light_count: 0,
            _padding: [0; 3],
        }
    }
}

/// Configuration for the renderer.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub color_format: TextureFormat,
    pub depth_format: TextureFormat,
    pub shadow_map_size: u32,
    pub shadow_cascade_count: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            color_format: TextureFormat::Bgra8UnormSrgb,
            depth_format: TextureFormat::Depth32Float,
            shadow_map_size: 2048,
            shadow_cascade_count: 3,
        }
    }
}

/// Main renderer that manages render passes and GPU resources.
pub struct Renderer {
    config: RendererConfig,
    depth_texture: Texture,
    depth_view: TextureView,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    lights_buffer: Buffer,
    light_bind_group: BindGroup,
    pbr_pipeline: RenderPipeline,
    shadow_mapper: ShadowMapper,
}

impl Renderer {
    /// Create a new renderer with the given device and configuration.
    pub fn new(device: &Device, _queue: &Queue, config: RendererConfig) -> Self {
        // Create depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(device, &config);

        // Create camera uniform buffer and bind group
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_layout = camera_bind_group_layout(device);
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create lights uniform buffer
        let lights_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Lights Uniform Buffer"),
            size: std::mem::size_of::<LightsUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shadow mapper
        let shadow_mapper = ShadowMapper::new(device, config.shadow_map_size, config.shadow_cascade_count);

        // Create light bind group with shadow map
        let light_layout = light_bind_group_layout(device);
        let shadow_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(shadow_mapper.view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // Create PBR pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/pbr.wgsl").into()),
        });

        let material_layout = material_bind_group_layout(device);
        let pbr_pipeline = RenderPipeline::new_pbr(
            device,
            &shader,
            config.color_format,
            config.depth_format,
            &camera_layout,
            &material_layout,
            &light_layout,
        );

        Self {
            config,
            depth_texture,
            depth_view,
            camera_buffer,
            camera_bind_group,
            lights_buffer,
            light_bind_group,
            pbr_pipeline,
            shadow_mapper,
        }
    }

    fn create_depth_texture(device: &Device, config: &RendererConfig) -> (Texture, TextureView) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.depth_format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        (texture, view)
    }

    /// Resize the renderer's internal buffers.
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        let (depth_texture, depth_view) = Self::create_depth_texture(device, &self.config);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
    }

    /// Update camera uniform on GPU.
    pub fn update_camera(&self, queue: &Queue, camera: &Camera, position: Vec3) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let uniform = CameraUniform::new(camera, position, aspect);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Update lights uniform on GPU.
    pub fn update_lights(&self, queue: &Queue, lights: &[(Light, Vec3, Vec3)]) {
        let mut uniform = LightsUniform::default();
        let count = lights.len().min(16);

        for (i, (light, pos, dir)) in lights.iter().take(count).enumerate() {
            uniform.lights[i] = LightUniform::from_light(light, *pos, *dir);
        }
        uniform.light_count = count as u32;

        queue.write_buffer(&self.lights_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Render a frame to the given surface texture view.
    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        target: &TextureView,
        renderables: &[(&Mesh, &PbrMaterial, Mat4)],
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Main color pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pbr_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            // Draw each renderable
            // Note: In a full implementation, mesh data would be uploaded to GPU buffers
            // and material bind groups would be created. For now we just iterate.
            for (_mesh, _material, _transform) in renderables {
                // In a complete implementation:
                // 1. Set vertex/index buffers from mesh GPU data
                // 2. Set material bind group
                // 3. Push transform via push constants or per-object uniform
                // 4. Draw indexed
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Get the shadow mapper for shadow pass operations.
    pub fn shadow_mapper(&self) -> &ShadowMapper {
        &self.shadow_mapper
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Get the depth texture view.
    pub fn depth_view(&self) -> &TextureView {
        &self.depth_view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_uniform_size() {
        // 3 Mat4s (3*64 = 192) + Vec4 (16) = 208 bytes
        let size = std::mem::size_of::<CameraUniform>();
        assert_eq!(size, 208);
    }

    #[test]
    fn test_light_uniform_size() {
        // 4 Vec4s (64) + u32 + padding
        let size = std::mem::size_of::<LightUniform>();
        assert_eq!(size, 80); // 64 + 16
    }

    #[test]
    fn test_lights_uniform_size() {
        // 16 lights * 80 + 16 padding = 1296
        let size = std::mem::size_of::<LightsUniform>();
        assert_eq!(size, 1296);
    }

    #[test]
    fn test_light_uniform_from_directional() {
        let light = Light::directional(Vec3::ONE, 1.0);
        let uniform = LightUniform::from_light(&light, Vec3::ZERO, Vec3::NEG_Y);
        assert_eq!(uniform.light_type, 0);
    }

    #[test]
    fn test_light_uniform_from_point() {
        let light = Light::point(Vec3::ONE, 1.0, 5.0);
        let uniform = LightUniform::from_light(&light, Vec3::new(1.0, 2.0, 3.0), Vec3::ZERO);
        assert_eq!(uniform.light_type, 1);
        assert_eq!(uniform.params[1], 5.0); // range
    }

    #[test]
    fn test_renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.shadow_map_size, 2048);
    }

    #[test]
    fn test_camera_uniform_creation() {
        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        let uniform = CameraUniform::new(&camera, Vec3::new(0.0, 5.0, 10.0), 16.0 / 9.0);
        assert_eq!(uniform.position[0], 0.0);
        assert_eq!(uniform.position[1], 5.0);
        assert_eq!(uniform.position[2], 10.0);
    }
}
