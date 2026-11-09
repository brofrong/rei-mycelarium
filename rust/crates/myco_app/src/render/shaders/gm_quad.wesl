// GameMaker-style textured quads (myco_app::render::batch::GmQuad).
//
// The mesh carries texel coordinates (not 0..1 UVs) and linear vertex colours. Sampling uses
// textureLoad, so the filter is chosen per draw (`params.y`: 0 nearest, 1 bilinear, as
// gpu_set_tex_filter) independent of the image's sampler.
//
// mode 0 (`params.x` = 0): texture x vertex colour (draw_sprite_ext & co).
// mode 1: one direction of sh_glow_blur: 9 alpha taps `params.zw` texels apart (Gaussian
// weights); the colour is the vertex colour, as the GLSL original.
//
// Colour space. Vertex colours arrive gamma-encoded (GameMaker colour / 255). Under SRGB_OUTPUT
// (camera::COMPOSITING, blending in sRGB space) texels are encoded before the bilinear mix and the
// product is written as is, so filtering, vertex-colour interpolation and blending all work on
// encoded values like GameMaker / LÖVE. Without SRGB_OUTPUT everything is decoded to linear.

struct FragIn {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}

@group(2) @binding(0) var tex: texture_2d<f32>;
@group(2) @binding(1) var tex_sampler: sampler;
@group(2) @binding(2) var<uniform> params: vec4<f32>;

fn texel(p: vec2<i32>) -> vec4<f32> {
    let d = vec2<i32>(textureDimensions(tex));
    var c = textureLoad(tex, clamp(p, vec2<i32>(0, 0), d - vec2<i32>(1, 1)), 0);
    @if(SRGB_OUTPUT)
    c = vec4<f32>(encode_srgb(c.rgb), c.a);
    return c;
}

fn decode_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn sample_at(uv: vec2<f32>) -> vec4<f32> {
    if params.y > 0.5 {
        let q = uv - vec2<f32>(0.5, 0.5);
        let i = vec2<i32>(floor(q));
        let f = q - floor(q);
        let a = mix(texel(i), texel(i + vec2<i32>(1, 0)), f.x);
        let b = mix(texel(i + vec2<i32>(0, 1)), texel(i + vec2<i32>(1, 1)), f.x);
        return mix(a, b, f.y);
    }
    return texel(vec2<i32>(floor(uv)));
}

fn encode_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fragment(in: FragIn) -> @location(0) vec4<f32> {
    var col = in.color;
    @if(!SRGB_OUTPUT)
    col = vec4<f32>(decode_srgb(col.rgb), col.a);
    var c: vec4<f32>;
    if params.x > 0.5 {
        let s = params.zw;
        var a = sample_at(in.uv).a * 0.2270270;
        a += (sample_at(in.uv + s).a + sample_at(in.uv - s).a) * 0.1945946;
        a += (sample_at(in.uv + s * 2.0).a + sample_at(in.uv - s * 2.0).a) * 0.1216216;
        a += (sample_at(in.uv + s * 3.0).a + sample_at(in.uv - s * 3.0).a) * 0.0540540;
        a += (sample_at(in.uv + s * 4.0).a + sample_at(in.uv - s * 4.0).a) * 0.0162162;
        c = vec4<f32>(col.rgb, a * col.a);
    } else {
        c = sample_at(in.uv) * col;
    }
    return c;
}
