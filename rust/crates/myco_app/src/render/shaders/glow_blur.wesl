// The mycelium glow (myco_app::render::mycelium_glow::GlowMaterial): sh_glow_blur and the passes
// around it. Lengths are in buffer texels; the quad's uv spans the whole source buffer.
//
// mode 0: copy the source.
// mode 1: thicken - the source plus eight copies `thick` texels away (every 45 degrees), added
//         and saturated, as eight draw_surface calls with bm_one/bm_one; texels a copy does not
//         cover add nothing.
// mode 2: sh_glow_blur - one direction of the nine-tap Gaussian over the alpha, `step` texels
//         apart; out = (white, a) as the GLSL original drawn with c_white.
// mode 3: composite (myc_glow_composite) - premultiplied for a bm_one/bm_one blend:
//         glow colour * seed alpha * glow strength, plus, where pulse rings run, pulse colour *
//         seed alpha * ring alpha * pulse strength. Colours are sRGB-encoded and added in encoded
//         space under SRGB_OUTPUT (camera::COMPOSITING), else handed over as linear values.
//
// Buffer reads use textureLoad with manual bilinear filtering (gpu_set_tex_filter(true)), edge
// clamped like a GameMaker surface.

struct FragIn {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct GlowParams {
    mode: vec4<f32>,
    glow: vec4<f32>,
    pulse: vec4<f32>,
    origin: vec4<f32>,
    rings: array<vec4<f32>, 8>,
}

@group(2) @binding(0) var tex: texture_2d<f32>;
@group(2) @binding(1) var tex_sampler: sampler;
@group(2) @binding(2) var<uniform> params: GlowParams;

fn texel(p: vec2<i32>, edge: bool) -> vec4<f32> {
    let d = vec2<i32>(textureDimensions(tex));
    if !edge && (p.x < 0 || p.y < 0 || p.x >= d.x || p.y >= d.y) {
        return vec4<f32>(0.0);
    }
    return textureLoad(tex, clamp(p, vec2<i32>(0, 0), d - vec2<i32>(1, 1)), 0);
}

fn bilinear(pos: vec2<f32>, edge: bool) -> vec4<f32> {
    let q = pos - vec2<f32>(0.5, 0.5);
    let i = vec2<i32>(floor(q));
    let f = q - floor(q);
    let a = mix(texel(i, edge), texel(i + vec2<i32>(1, 0), edge), f.x);
    let b = mix(texel(i + vec2<i32>(0, 1), edge), texel(i + vec2<i32>(1, 1), edge), f.x);
    return mix(a, b, f.y);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// myc_pulse_ring's two strips: 0 at r - w rising to a at r, falling to 0 at r + w
fn ring(d: f32, r: f32, w: f32, a: f32) -> vec2<f32> {
    let rin = max(0.0, r - w);
    let rout = r + w;
    if d < rin || d > rout {
        return vec2<f32>(0.0, 0.0);
    }
    var v: f32;
    if d <= r {
        if r > rin {
            v = a * (d - rin) / (r - rin);
        } else {
            v = a;
        }
    } else {
        v = a * (rout - d) / (rout - r);
    }
    // (covered, alpha)
    return vec2<f32>(1.0, v);
}

@fragment
fn fragment(in: FragIn) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(tex));
    let pos = in.uv * size;
    let mode = params.mode.x;
    if mode < 0.5 {
        return bilinear(pos, true);
    }
    if mode < 1.5 {
        let thick = params.mode.y;
        var sum = bilinear(pos, false);
        for (var k = 0; k < 8; k++) {
            let ang = radians(f32(k) * 45.0);
            let off = vec2<f32>(cos(ang) * thick, -sin(ang) * thick);
            sum += bilinear(pos - off, false);
        }
        return min(sum, vec4<f32>(1.0));
    }
    if mode < 2.5 {
        let s = params.mode.zw;
        var a = bilinear(pos, true).a * 0.2270270;
        a += (bilinear(pos + s, true).a + bilinear(pos - s, true).a) * 0.1945946;
        a += (bilinear(pos + s * 2.0, true).a + bilinear(pos - s * 2.0, true).a) * 0.1216216;
        a += (bilinear(pos + s * 3.0, true).a + bilinear(pos - s * 3.0, true).a) * 0.0540540;
        a += (bilinear(pos + s * 4.0, true).a + bilinear(pos - s * 4.0, true).a) * 0.0162162;
        return vec4<f32>(1.0, 1.0, 1.0, a);
    }
    // composite
    let sa = bilinear(pos, true).a;
    var v = params.glow.rgb * (sa * params.glow.a);
    let n = i32(params.origin.w);
    if n > 0 {
        let d = length(pos - params.origin.xy);
        var covered = 0.0;
        var ra = 0.0;
        for (var i = 0; i < 8; i++) {
            if i < n {
                let r = ring(d, params.rings[i].x, params.origin.z, params.rings[i].y);
                covered = max(covered, r.x);
                ra += r.y;
            }
        }
        ra = min(ra, 1.0);
        v += params.pulse.rgb * (covered * sa * ra * params.pulse.a);
    }
    v = min(v, vec3<f32>(1.0));
    var out = v;
    @if(!SRGB_OUTPUT)
    out = srgb_to_linear(v);
    return vec4<f32>(out, 1.0);
}
