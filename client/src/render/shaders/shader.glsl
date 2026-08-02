#version 300 es

precision highp float;

in vec2 uv;
out vec4 fragColor;

#define MAX_BUBBLES 256

uniform sampler2D u_position_radius_texture;
uniform sampler2D u_color_pressure_texture;

uniform int u_bubble_count;
uniform float u_texture_width;

uniform vec2 u_resolution;
uniform vec2 u_camera_pos;
uniform float u_zoom;

// --- COLOR SPACE CONVERSIONS (Oklab -> Linear -> sRGB Only) ---
vec3 linear_to_srgb(vec3 c) { return pow(clamp(c, 0.0, 1.0), vec3(1.0 / 2.2)); }

vec3 oklab_to_rgb(vec3 lab) {
    float l_ = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
    float m_ = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
    float s_ = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;

    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;

    return vec3(
        +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    );
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

vec4 read_position_radius(int index) {
    return texelFetch(u_position_radius_texture, ivec2(index, 0), 0);
}

vec4 read_color_pressure(int index) {
    return texelFetch(u_color_pressure_texture, ivec2(index, 0), 0);
}

void main() {
    vec2 world_pos = (gl_FragCoord.xy - u_resolution * 0.5) / u_zoom + u_camera_pos;
    if (u_bubble_count == 0) {
        fragColor = vec4(0.015, 0.02, 0.04, 1.0); return;
    }

    // --- SINGLE PASS: Topology (d1/d2/d3, outer_sdf) AND color weighting
    // together. This works because color weighting now only depends on the absolute
    // distance of each bubble to itself, not on the final d1. As a result, d1
    // no longer needs to be fixed during color accumulation.
    // Culling now applies to BOTH purposes, not just color: a bubble that is
    // farther away than CULL_RADIUS can never be the nearest surface (d1) for a
    // pixel that is anywhere near any geometry—and if the pixel is far out in
    // empty space, the exact d1 value is irrelevant for rendering anyway.
    float d1 = 1e5; float d2 = 1e5; float d3 = 1e5;
    float outer_sdf = 1e5;

    vec3 lab_accum = vec3(0.0);
    float light_accum = 0.0;
    float radius_accum = 0.0;
    float weight_sum = 0.0001;

    const float BLEED_RADIUS = 70.0;
    const float CULL_RADIUS = BLEED_RADIUS * 5.5;

    for (int i = 0; i < MAX_BUBBLES; i++) {
        if (i >= u_bubble_count) break;

        vec4 position_radius = read_position_radius(i);
        float radius = position_radius.z;
        float d = length(world_pos - position_radius.xy) - radius;

        // EARLY DISTANCE CULLING - now shared for topology AND color,
        // and completely eliminates the second (redundant) position_radius fetch.
        if (d > CULL_RADIUS) continue;

        // Voronoi distance sorting (previously: separate, unfiltered pass)
        outer_sdf = smin(outer_sdf, d, 35.0);
        if (d < d1) {
            d3 = d2; d2 = d1; d1 = d;
        } else if (d < d2) {
            d3 = d2; d2 = d;
        } else if (d < d3) {
            d3 = d;
        }

        // Farbgewichtung
        vec4 color_pressure = read_color_pressure(i);
        mediump float light = position_radius.w;
        mediump vec3 lab_col = color_pressure.rgb; // schon Oklab, kein pow() noetig

        mediump float w = exp(-max(0.0, d) / BLEED_RADIUS);
        // WINDOW FUNCTION: Smoothly fade weight to 0 over the last 20% of the cull distance
        w *= 1.0 - smoothstep(CULL_RADIUS * 0.8, CULL_RADIUS, d);

        lab_accum += lab_col * w;
        light_accum += light * w;
        radius_accum += radius * w;
        weight_sum += w;
    }

    vec3 avg_lab = lab_accum / weight_sum;
    float avg_light = light_accum / weight_sum;
    float avg_radius = radius_accum / weight_sum;
    vec3 blended_srgb = linear_to_srgb(oklab_to_rgb(avg_lab));

    // --- DYNAMIC MEMBRANE MATH ---
    const float MIN_WALL_WIDTH = 3.0;
    const float MAX_WALL_WIDTH = 12.0;
    const float REF_MIN_RADIUS = 20.0;
    const float REF_MAX_RADIUS = 120.0;

    float radius_t = clamp((avg_radius - REF_MIN_RADIUS) / (REF_MAX_RADIUS - REF_MIN_RADIUS), 0.0, 1.0);
    float dynamic_wall_width = mix(MIN_WALL_WIDTH, MAX_WALL_WIDTH, radius_t);
    float raw_wall = (d2 - d1) * 0.5;
    float wall_junction_blend = (d3 - d2) * 0.5;

    float junction_curve = wall_junction_blend + 8.0 + smoothstep(0.0, 15.0, raw_wall) * 100.0;
    float internal_wall = smin(raw_wall, junction_curve, 15.0);
    float internal_membrane = smoothstep(dynamic_wall_width, 0.0, internal_wall);
    float near_cluster = smoothstep(dynamic_wall_width, 0.0, outer_sdf);
    internal_membrane *= near_cluster;
    float outer_membrane = smoothstep(dynamic_wall_width, 0.0, abs(outer_sdf));
    float total_membrane = max(outer_membrane, internal_membrane);
    float inside_mask = smoothstep(1.5, -1.5, outer_sdf);

    // --- SHADING & COLORING ---
    vec3 background = vec3(0.015, 0.02, 0.04);
    const float DEPTH_FALLOFF = 45.0;

    float edge_dist = max(0.0, -outer_sdf);
    mediump float cell_depth = 0.25 + 0.75 * exp(-edge_dist / DEPTH_FALLOFF);
    mediump vec3 interior_color = blended_srgb * (0.2 + 0.8 * avg_light) * cell_depth;
    mediump vec3 wall_color = vec3(0.95, 0.98, 1.0) * (0.25 + 0.75 * avg_light);
    mediump vec3 final_color = mix(background, interior_color, inside_mask);
    final_color = mix(final_color, wall_color, total_membrane * 0.95);

    // --- HALO / GLOW ---
    const float HALO_BOOST = 1.4;
    float halo_scale = clamp(avg_radius / 120.0, 0.2, 3.0);
    float dist = max(0.0, outer_sdf);
    mediump float inner_halo = exp(-dist * (0.02 / halo_scale)) * 0.6;
    mediump float outer_halo = exp(-dist * (0.005 / halo_scale)) * 0.4;
    mediump float total_halo = (inner_halo + outer_halo) * avg_light * HALO_BOOST;

    final_color += blended_srgb * total_halo;

    fragColor = vec4(final_color, 1.0);
}