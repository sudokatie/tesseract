//! Shadow mapping with cascaded shadow maps for directional lights.

use glam::{Mat4, Vec3, Vec4};
use wgpu::{
    CommandEncoder, Device, Extent3d, LoadOp, Operations, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

use super::{Camera, Light, LightKind, Projection};

/// Configuration for shadow cascade splits.
#[derive(Debug, Clone)]
pub struct CascadeConfig {
    /// Number of cascades (typically 3-4).
    pub cascade_count: u32,
    /// Split scheme lambda (0 = uniform, 1 = logarithmic).
    pub split_lambda: f32,
    /// Shadow map resolution per cascade.
    pub resolution: u32,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            cascade_count: 3,
            split_lambda: 0.5,
            resolution: 2048,
        }
    }
}

/// A single cascade in the cascaded shadow map.
#[derive(Debug, Clone)]
pub struct ShadowCascade {
    /// Near plane of this cascade in view space.
    pub near: f32,
    /// Far plane of this cascade in view space.
    pub far: f32,
    /// Light-space view-projection matrix.
    pub view_projection: Mat4,
    /// Cascade index (0 = nearest).
    pub index: u32,
}

/// Cascaded shadow map calculator.
#[derive(Debug, Clone)]
pub struct CascadedShadowMap {
    cascades: Vec<ShadowCascade>,
    config: CascadeConfig,
}

impl CascadedShadowMap {
    /// Create a new cascaded shadow map calculator.
    pub fn new(config: CascadeConfig) -> Self {
        Self {
            cascades: Vec::with_capacity(config.cascade_count as usize),
            config,
        }
    }

    /// Calculate cascade splits and light matrices.
    pub fn calculate_cascades(
        &mut self,
        camera: &Camera,
        camera_position: Vec3,
        light_direction: Vec3,
        near: f32,
        far: f32,
    ) {
        self.cascades.clear();

        let cascade_count = self.config.cascade_count as usize;
        let lambda = self.config.split_lambda;

        // Calculate split distances
        let mut splits = Vec::with_capacity(cascade_count + 1);
        splits.push(near);

        for i in 1..=cascade_count {
            let p = i as f32 / cascade_count as f32;
            // Practical split scheme (blend of uniform and logarithmic)
            let log_split = near * (far / near).powf(p);
            let uniform_split = near + (far - near) * p;
            let split = lambda * log_split + (1.0 - lambda) * uniform_split;
            splits.push(split);
        }

        // Get camera inverse view matrix (world to view space inverse)
        // We build the view matrix directly from position looking down -Z
        let camera_view = Mat4::look_at_rh(camera_position, camera_position + Vec3::NEG_Z, Vec3::Y);
        let inv_camera_view = camera_view.inverse();

        for i in 0..cascade_count {
            let cascade_near = splits[i];
            let cascade_far = splits[i + 1];

            // Calculate frustum corners for this cascade
            let frustum_corners = self.calculate_frustum_corners(
                camera,
                &inv_camera_view,
                cascade_near,
                cascade_far,
            );

            // Calculate light view-projection matrix
            let view_projection = self.calculate_light_matrix(&frustum_corners, light_direction);

            self.cascades.push(ShadowCascade {
                near: cascade_near,
                far: cascade_far,
                view_projection,
                index: i as u32,
            });
        }
    }

    fn calculate_frustum_corners(
        &self,
        camera: &Camera,
        inv_camera_view: &Mat4,
        near: f32,
        far: f32,
    ) -> [Vec3; 8] {
        let aspect = 16.0 / 9.0; // TODO: pass actual aspect

        let (tan_half_fov_y, tan_half_fov_x) = match camera.projection {
            Projection::Perspective { fov, .. } => {
                let tan_half_fov = (fov / 2.0).tan();
                (tan_half_fov, tan_half_fov * aspect)
            }
            Projection::Orthographic { size, .. } => {
                // For ortho, use direct size
                let half_height = size / 2.0;
                let half_width = half_height * aspect;
                (half_height / far, half_width / far)
            }
        };

        let near_height = near * tan_half_fov_y;
        let near_width = near * tan_half_fov_x;
        let far_height = far * tan_half_fov_y;
        let far_width = far * tan_half_fov_x;

        // Frustum corners in view space (camera looking down -Z)
        let corners_view = [
            // Near plane
            Vec3::new(-near_width, -near_height, -near),
            Vec3::new(near_width, -near_height, -near),
            Vec3::new(near_width, near_height, -near),
            Vec3::new(-near_width, near_height, -near),
            // Far plane
            Vec3::new(-far_width, -far_height, -far),
            Vec3::new(far_width, -far_height, -far),
            Vec3::new(far_width, far_height, -far),
            Vec3::new(-far_width, far_height, -far),
        ];

        // Transform to world space
        let mut corners_world = [Vec3::ZERO; 8];
        for (i, corner) in corners_view.iter().enumerate() {
            let world = *inv_camera_view * Vec4::new(corner.x, corner.y, corner.z, 1.0);
            corners_world[i] = Vec3::new(world.x, world.y, world.z);
        }

        corners_world
    }

    fn calculate_light_matrix(&self, frustum_corners: &[Vec3; 8], light_direction: Vec3) -> Mat4 {
        // Calculate frustum center
        let center = frustum_corners.iter().fold(Vec3::ZERO, |acc, c| acc + *c) / 8.0;

        // Light view matrix (looking along light direction)
        let light_dir = light_direction.normalize();
        let up = if light_dir.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let light_view = Mat4::look_at_rh(center - light_dir * 100.0, center, up);

        // Transform frustum corners to light space and find bounds
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for corner in frustum_corners {
            let light_space = light_view * Vec4::new(corner.x, corner.y, corner.z, 1.0);
            let p = Vec3::new(light_space.x, light_space.y, light_space.z);
            min = min.min(p);
            max = max.max(p);
        }

        // Expand bounds slightly to avoid edge artifacts
        let padding = 1.0;
        min -= Vec3::splat(padding);
        max += Vec3::splat(padding);

        // Create orthographic projection
        let light_projection = Mat4::orthographic_rh(min.x, max.x, min.y, max.y, -max.z - 100.0, -min.z);

        light_projection * light_view
    }

    /// Get the calculated cascades.
    pub fn cascades(&self) -> &[ShadowCascade] {
        &self.cascades
    }

    /// Get cascade for a given view-space depth.
    pub fn get_cascade_for_depth(&self, depth: f32) -> Option<&ShadowCascade> {
        self.cascades.iter().find(|c| depth >= c.near && depth < c.far)
    }
}

/// Shadow map texture manager.
pub struct ShadowMapper {
    texture: Texture,
    view: TextureView,
    cascade_views: Vec<TextureView>,
    resolution: u32,
    cascade_count: u32,
    cascaded_shadow_map: CascadedShadowMap,
}

impl ShadowMapper {
    /// Create a new shadow mapper.
    pub fn new(device: &Device, resolution: u32, cascade_count: u32) -> Self {
        // Create shadow map texture array (one layer per cascade)
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Shadow Map"),
            size: Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: cascade_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create view for all cascades (for sampling in shader)
        let view = texture.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Create individual views for each cascade (for rendering)
        let mut cascade_views = Vec::with_capacity(cascade_count as usize);
        for i in 0..cascade_count {
            let cascade_view = texture.create_view(&TextureViewDescriptor {
                base_array_layer: i,
                array_layer_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            });
            cascade_views.push(cascade_view);
        }

        let config = CascadeConfig {
            cascade_count,
            resolution,
            ..Default::default()
        };

        Self {
            texture,
            view,
            cascade_views,
            resolution,
            cascade_count,
            cascaded_shadow_map: CascadedShadowMap::new(config),
        }
    }

    /// Update cascade matrices for a given camera and light.
    pub fn update_cascades(
        &mut self,
        camera: &Camera,
        camera_position: Vec3,
        light: &Light,
        light_direction: Vec3,
    ) {
        if !matches!(light.kind, LightKind::Directional) {
            return; // Cascaded shadows only for directional lights
        }

        let (near, far) = match camera.projection {
            Projection::Perspective { near, far, .. } => (near, far),
            Projection::Orthographic { near, far, .. } => (near, far),
        };

        self.cascaded_shadow_map.calculate_cascades(
            camera,
            camera_position,
            light_direction,
            near,
            far,
        );
    }

    /// Render shadow maps for all cascades.
    pub fn render_shadows<F>(&self, encoder: &mut CommandEncoder, mut draw_fn: F)
    where
        F: FnMut(&mut wgpu::RenderPass, &ShadowCascade),
    {
        for (i, cascade) in self.cascaded_shadow_map.cascades().iter().enumerate() {
            let cascade_view = &self.cascade_views[i];

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Shadow Pass Cascade {}", i)),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: cascade_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            draw_fn(&mut render_pass, cascade);
        }
    }

    /// Get the shadow map texture view (for binding to shaders).
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// Get the underlying texture (for advanced operations).
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Get the cascaded shadow map data.
    pub fn cascaded_shadow_map(&self) -> &CascadedShadowMap {
        &self.cascaded_shadow_map
    }

    /// Get shadow map resolution.
    pub fn resolution(&self) -> u32 {
        self.resolution
    }

    /// Get number of cascades.
    pub fn cascade_count(&self) -> u32 {
        self.cascade_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_config_default() {
        let config = CascadeConfig::default();
        assert_eq!(config.cascade_count, 3);
        assert_eq!(config.resolution, 2048);
        assert!((config.split_lambda - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cascaded_shadow_map_new() {
        let config = CascadeConfig::default();
        let csm = CascadedShadowMap::new(config);
        assert!(csm.cascades().is_empty());
    }

    #[test]
    fn test_cascade_calculation() {
        let config = CascadeConfig {
            cascade_count: 3,
            split_lambda: 0.5,
            resolution: 1024,
        };
        let mut csm = CascadedShadowMap::new(config);

        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        let light_dir = Vec3::new(1.0, -1.0, 0.5).normalize();

        csm.calculate_cascades(&camera, Vec3::ZERO, light_dir, 0.1, 100.0);

        assert_eq!(csm.cascades().len(), 3);
        assert!(csm.cascades()[0].near < csm.cascades()[0].far);
        assert!(csm.cascades()[1].near < csm.cascades()[1].far);
        assert!(csm.cascades()[2].near < csm.cascades()[2].far);
    }

    #[test]
    fn test_cascade_splits_continuous() {
        let config = CascadeConfig {
            cascade_count: 4,
            split_lambda: 0.5,
            resolution: 1024,
        };
        let mut csm = CascadedShadowMap::new(config);

        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 1.0, 500.0);
        csm.calculate_cascades(&camera, Vec3::ZERO, Vec3::NEG_Y, 1.0, 500.0);

        // Verify cascades are continuous
        for i in 1..csm.cascades().len() {
            let prev_far = csm.cascades()[i - 1].far;
            let curr_near = csm.cascades()[i].near;
            assert!((prev_far - curr_near).abs() < 0.001);
        }
    }

    #[test]
    fn test_get_cascade_for_depth() {
        let config = CascadeConfig {
            cascade_count: 3,
            split_lambda: 0.5,
            resolution: 1024,
        };
        let mut csm = CascadedShadowMap::new(config);

        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        csm.calculate_cascades(&camera, Vec3::ZERO, Vec3::NEG_Y, 0.1, 100.0);

        // Near depth should be in cascade 0
        let cascade = csm.get_cascade_for_depth(0.5);
        assert!(cascade.is_some());
        assert_eq!(cascade.unwrap().index, 0);

        // Far depth should be in last cascade
        let cascade = csm.get_cascade_for_depth(90.0);
        assert!(cascade.is_some());
        assert_eq!(cascade.unwrap().index, 2);
    }

    #[test]
    fn test_cascade_covers_range() {
        let config = CascadeConfig {
            cascade_count: 3,
            split_lambda: 0.5,
            resolution: 1024,
        };
        let mut csm = CascadedShadowMap::new(config);

        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        csm.calculate_cascades(&camera, Vec3::ZERO, Vec3::NEG_Y, 0.1, 100.0);

        // First cascade should start near the near plane
        assert!(csm.cascades()[0].near <= 0.2);
        // Last cascade should end near the far plane
        assert!(csm.cascades()[2].far >= 99.0);
    }

    #[test]
    fn test_light_matrix_is_valid() {
        let config = CascadeConfig {
            cascade_count: 1,
            split_lambda: 0.5,
            resolution: 1024,
        };
        let mut csm = CascadedShadowMap::new(config);

        let camera = Camera::perspective(std::f32::consts::FRAC_PI_4, 1.0, 100.0);
        // Camera at reasonable position looking forward
        csm.calculate_cascades(
            &camera,
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
            1.0,
            100.0,
        );

        let cascade = &csm.cascades()[0];
        // Matrix should not be identity (has actual transformation)
        assert!(cascade.view_projection != Mat4::IDENTITY);
    }
}
