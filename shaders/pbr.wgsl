// PBR Shader - Cook-Torrance BRDF

// Camera uniform
struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> camera: Camera;

// Material uniform
struct Material {
    albedo: vec4<f32>,
    metallic_roughness: vec4<f32>, // x: metallic, y: roughness
    emissive: vec4<f32>,
    flags: vec4<u32>, // x: has_albedo, y: has_normal, z: has_mr, w: has_ao
}

@group(1) @binding(0) var<uniform> material: Material;
@group(1) @binding(1) var albedo_texture: texture_2d<f32>;
@group(1) @binding(2) var albedo_sampler: sampler;
@group(1) @binding(3) var normal_texture: texture_2d<f32>;
@group(1) @binding(4) var normal_sampler: sampler;

// Light uniform
const MAX_LIGHTS: u32 = 16u;

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    params: vec4<f32>, // x: kind (0=dir, 1=point, 2=spot, 3=ambient), y: intensity, z: range, w: angle
}

struct Lights {
    count: u32,
    _padding: vec3<u32>,
    lights: array<Light, MAX_LIGHTS>,
}

@group(2) @binding(0) var<uniform> lights: Lights;
@group(2) @binding(1) var shadow_map: texture_depth_2d;
@group(2) @binding(2) var shadow_sampler: sampler_comparison;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

// Model matrix passed as instance data or push constant
// For simplicity, using identity here - real impl would use instance buffer
var<private> model: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = model * vec4<f32>(input.position, 1.0);
    output.world_position = world_pos.xyz;
    output.clip_position = camera.projection * camera.view * world_pos;
    
    let normal_matrix = mat3x3<f32>(
        model[0].xyz,
        model[1].xyz,
        model[2].xyz
    );
    output.world_normal = normalize(normal_matrix * input.normal);
    output.tangent = normalize(normal_matrix * input.tangent.xyz);
    output.bitangent = cross(output.world_normal, output.tangent) * input.tangent.w;
    output.uv = input.uv;
    
    return output;
}

// PBR functions
const PI: f32 = 3.14159265359;

// Normal Distribution Function (GGX/Trowbridge-Reitz)
fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(n, h), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;
    
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

// Geometry function (Smith's method with GGX)
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

// Fresnel (Schlick approximation)
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    var albedo = material.albedo.rgb;
    if material.flags.x != 0u {
        albedo = textureSample(albedo_texture, albedo_sampler, input.uv).rgb;
    }
    
    var normal = input.world_normal;
    if material.flags.y != 0u {
        let tangent_normal = textureSample(normal_texture, normal_sampler, input.uv).rgb * 2.0 - 1.0;
        let tbn = mat3x3<f32>(input.tangent, input.bitangent, input.world_normal);
        normal = normalize(tbn * tangent_normal);
    }
    
    let metallic = material.metallic_roughness.x;
    let roughness = material.metallic_roughness.y;
    
    // View direction
    let v = normalize(camera.position - input.world_position);
    
    // Reflectance at normal incidence
    var f0 = vec3<f32>(0.04);
    f0 = mix(f0, albedo, metallic);
    
    // Accumulate lighting
    var lo = vec3<f32>(0.0);
    
    for (var i = 0u; i < lights.count && i < MAX_LIGHTS; i++) {
        let light = lights.lights[i];
        let kind = u32(light.params.x);
        let intensity = light.params.y;
        
        var l: vec3<f32>;
        var attenuation: f32 = 1.0;
        
        if kind == 0u {
            // Directional light
            l = normalize(-light.direction.xyz);
        } else if kind == 1u {
            // Point light
            let light_vec = light.position.xyz - input.world_position;
            let distance = length(light_vec);
            l = normalize(light_vec);
            let range = light.params.z;
            attenuation = max(0.0, 1.0 - distance / range);
            attenuation *= attenuation;
        } else if kind == 2u {
            // Spot light
            let light_vec = light.position.xyz - input.world_position;
            let distance = length(light_vec);
            l = normalize(light_vec);
            let range = light.params.z;
            let spot_angle = light.params.w;
            
            let theta = dot(l, normalize(-light.direction.xyz));
            let epsilon = 0.1;
            let spot_intensity = clamp((theta - cos(spot_angle)) / epsilon, 0.0, 1.0);
            
            attenuation = max(0.0, 1.0 - distance / range) * spot_intensity;
            attenuation *= attenuation;
        } else {
            // Ambient light - handled separately
            continue;
        }
        
        let h = normalize(v + l);
        let radiance = light.color.rgb * intensity * attenuation;
        
        // Cook-Torrance BRDF
        let ndf = distribution_ggx(normal, h, roughness);
        let g = geometry_smith(normal, v, l, roughness);
        let f = fresnel_schlick(max(dot(h, v), 0.0), f0);
        
        let numerator = ndf * g * f;
        let denominator = 4.0 * max(dot(normal, v), 0.0) * max(dot(normal, l), 0.0) + 0.0001;
        let specular = numerator / denominator;
        
        let ks = f;
        var kd = vec3<f32>(1.0) - ks;
        kd *= 1.0 - metallic;
        
        let n_dot_l = max(dot(normal, l), 0.0);
        lo += (kd * albedo / PI + specular) * radiance * n_dot_l;
    }
    
    // Ambient (from ambient lights)
    var ambient = vec3<f32>(0.03) * albedo;
    for (var i = 0u; i < lights.count && i < MAX_LIGHTS; i++) {
        let light = lights.lights[i];
        if u32(light.params.x) == 3u {
            ambient += light.color.rgb * light.params.y * albedo;
        }
    }
    
    // Emissive
    let emissive = material.emissive.rgb;
    
    // Final color
    let color = ambient + lo + emissive;
    
    // Tone mapping (Reinhard)
    let mapped = color / (color + vec3<f32>(1.0));
    
    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma_corrected, 1.0);
}

// Shadow vertex shader
@vertex
fn vs_shadow(input: VertexInput) -> @builtin(position) vec4<f32> {
    // Light space transform would come from uniform
    // Simplified: just transform by model
    return model * vec4<f32>(input.position, 1.0);
}
