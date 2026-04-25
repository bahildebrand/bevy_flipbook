#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    forward_io::VertexOutput,
    mesh_view_bindings::globals,
}

struct VatSettings {
    bounds_min: vec3<f32>,
    frame_count: u32,       // total animation frames (from remap_info "Frames")
    bounds_max: vec3<f32>,
    y_resolution: f32,      // actual texture pixel height (frame_count * 2 for pos+normals)
    fps: f32,
}

struct VatSlot {
    time_offset: f32,
    clip_start_frame: f32,
    clip_frame_count: f32,
    rate: f32
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var vat_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var vat_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> vat: VatSettings;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var<storage, read> slots: array<VatSlot>;


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
    let tag = mesh_functions::get_tag(in.instance_index);
    let slot = slots[tag];

    let start_frame = slot.clip_start_frame;
    let frame_count = slot.clip_frame_count;
    let time_offset = slot.time_offset;

    // Compute which frame to sample, looping within the clip
    let elapsed_frames = (globals.time - time_offset) * vat.fps * slot.rate;

    let frame = start_frame + (elapsed_frames % frame_count);

    let curr_frame = floor(frame);
    // Wrap next frame within the clip, not the whole texture
    let next_in_clip = (curr_frame - start_frame + 1.0) % frame_count;
    let next_frame = start_frame + next_in_clip;
    let blend = fract(frame);

    let frame_step = 1.0 / vat.y_resolution;
    // Sample at pixel centers (+0.5) to avoid bleeding into adjacent rows
    let uv_curr = vec2<f32>(in.uv_vat.x, (curr_frame + 0.5) * frame_step);
    let uv_next = vec2<f32>(in.uv_vat.x, (next_frame + 0.5) * frame_step);

    let encoded_curr = textureSampleLevel(vat_texture, vat_sampler, uv_curr, 0.0).rgb;
    let encoded_next = textureSampleLevel(vat_texture, vat_sampler, uv_next, 0.0).rgb;
    let encoded = mix(encoded_curr, encoded_next, blend);

    // Normals are packed in the bottom half of the texture (V + 0.5)
    let norm_curr = textureSampleLevel(vat_texture, vat_sampler, uv_curr + vec2<f32>(0.0, 0.5), 0.0).rgb;
    let norm_next = textureSampleLevel(vat_texture, vat_sampler, uv_next + vec2<f32>(0.0, 0.5), 0.0).rgb;
    // Decode [0,1] -> [-1,1] and apply same Blender->Bevy axis swap
    var n = normalize(mix(norm_curr, norm_next, blend) * 2.0 - 1.0);
    let animated_normal = vec3<f32>(n.x, n.z, -n.y);

    // Decode normalized [0,1] back to object-space offset
    let range = vat.bounds_max - vat.bounds_min;
    let blender_offset = vat.bounds_min + encoded * range;

    // Blender (Z-up RH) -> Bevy (Y-up RH):
    //   Bevy.x =  Blender.x  (R)
    //   Bevy.y =  Blender.z  (B)
    //   Bevy.z = -Blender.y  (-G)
    let offset = vec3<f32>(blender_offset.x, blender_offset.z, -blender_offset.y);

    // VAT stores offsets from rest pose
    let animated_position = in.position + offset;

    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);

    var out: VertexOutput;
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(animated_position, 1.0),
    );
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(animated_normal, in.instance_index);

#ifdef VERTEX_UVS_A
    out.uv = in.uv;
#endif

    return out;
}
