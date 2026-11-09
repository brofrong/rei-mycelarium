// sh_blackmetal (myco_app::render::post::BwQuad): the BLACK METAL post effect and the bright
// pass its glow is built from. One switch covers both: bright 0 tone maps the frame, bright 1
// keeps only what is above the threshold.
//
// Like gm_quad.wesl the mesh carries texel coordinates and gamma-encoded vertex colours; the
// texture is read with textureLoad (params.z: 0 nearest, 1 bilinear). Under SRGB_OUTPUT
// (camera::COMPOSITING) texels are encoded first, so the luma, contrast and gamma work on the
// encoded values as in GameMaker / LÖVE, and the result is written as is.

struct FragIn {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}

@group(2) @binding(0) var tex: texture_2d<f32>;
@group(2) @binding(1) var tex_sampler: sampler;
// x: u_gain, y: u_pivot, z: u_contrast, w: u_gamma
@group(2) @binding(2) var<uniform> tone: vec4<f32>;
// x: u_threshold, y: u_bright, z: bilinear filter
@group(2) @binding(3) var<uniform> extra: vec4<f32>;

fn encode_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn decode_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn texel(p: vec2<i32>) -> vec4<f32> {
    let d = vec2<i32>(textureDimensions(tex));
    var c = textureLoad(tex, clamp(p, vec2<i32>(0, 0), d - vec2<i32>(1, 1)), 0);
    @if(SRGB_OUTPUT)
    c = vec4<f32>(encode_srgb(c.rgb), c.a);
    return c;
}

fn sample_at(uv: vec2<f32>) -> vec4<f32> {
    if extra.z > 0.5 {
        let q = uv - vec2<f32>(0.5, 0.5);
        let i = vec2<i32>(floor(q));
        let f = q - floor(q);
        let a = mix(texel(i), texel(i + vec2<i32>(1, 0)), f.x);
        let b = mix(texel(i + vec2<i32>(0, 1)), texel(i + vec2<i32>(1, 1)), f.x);
        return mix(a, b, f.y);
    }
    return texel(vec2<i32>(floor(uv)));
}

@fragment
fn fragment(in: FragIn) -> @location(0) vec4<f32> {
    var col = in.color;
    @if(!SRGB_OUTPUT)
    col = vec4<f32>(decode_srgb(col.rgb), col.a);
    let c = col * sample_at(in.uv);
    // Rec. 601 luma
    var luma = dot(c.rgb, vec3<f32>(0.299, 0.587, 0.114));
    luma = luma * tone.x;
    luma = clamp((luma - tone.y) * tone.z + tone.y, 0.0, 1.0);
    let o = pow(luma, tone.w);
    let above = max(0.0, o - extra.x) / max(0.001, 1.0 - extra.x);
    var rgb = vec3<f32>(mix(o, above, extra.y));
    @if(!SRGB_OUTPUT)
    rgb = decode_srgb(rgb);
    return vec4<f32>(rgb, c.a);
}
