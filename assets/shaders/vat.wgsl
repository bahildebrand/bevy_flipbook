#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    forward_io::VertexOutput,
}

struct VatSettings {
    bounds_min: vec3<f32>,
    total_frames: f32,
    bounds_max: vec3<f32>,
    fps: f32,
    current_time: f32,
    clip_start_frame: f32,
    clip_frame_count: f32,
    _padding: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var vat_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var vat_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> vat: VatSettings;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // UV1: x encodes the vertex's column in the VAT texture (baked by OpenVAT)
    @location(3) uv_vat: vec2<f32>,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let frame = vat.clip_start_frame + (vat.current_time * vat.fps % vat.clip_frame_count);
    let vat_uv = vec2<f32>(in.uv_vat.x, (frame + 0.5) / vat.total_frames);

    let encoded = textureSampleLevel(vat_texture, vat_sampler, vat_uv, 0.0);

    // Decode normalized [0,1] back to object-space position
    let animated_position = vat.bounds_min + encoded.xyz * (vat.bounds_max - vat.bounds_min);

    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);

    var out: VertexOutput;
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(animated_position, 1.0),
    );
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(in.normal, in.instance_index);

#ifdef VERTEX_UVS_A
    out.uv = in.uv;
#endif

    return out;
}
