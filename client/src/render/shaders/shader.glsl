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

// --- COLOR SPACE CONVERSIONS (sRGB <-> Linear <-> Oklab) ---
vec3 srgb_to_linear(vec3 c) { return pow(c, vec3(2.2)); }
vec3 linear_to_srgb(vec3 c) { return pow(clamp(c, 0.0, 1.0), vec3(1.0 / 2.2)); }

vec3 rgb_to_oklab(vec3 c) {
    float l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    float m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    float s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    float l_ = pow(max(0.0, l), 1.0 / 3.0);
    float m_ = pow(max(0.0, m), 1.0 / 3.0);
    float s_ = pow(max(0.0, s), 1.0 / 3.0);

    return vec3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    );
}

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

vec4 read_position_radius(int index)
{
    return texelFetch(
        u_position_radius_texture,
        ivec2(index, 0),
        0
    );
}

vec4 read_color_pressure(int index)
{
    return texelFetch(
        u_color_pressure_texture,
        ivec2(index, 0),
        0
    );
}

// void main()
// {    
//     vec4 p = read_position_radius(0);
//     vec4 c = read_color_pressure(0);

//     fragColor = vec4(
//         c.r,
//         c.g,
//         c.b,
//         1.0
//     );
// }

void main()
{
    vec2 world_pos = (gl_FragCoord.xy - u_resolution * 0.5) / u_zoom + u_camera_pos;
    if (u_bubble_count == 0) { 
        fragColor = vec4(0.015, 0.02, 0.04, 1.0); return;
    }
    
    // --- PASS 1: Distance metrics ---
    float d1 = 1e5; float d2 = 1e5; float d3 = 1e5;
    float outer_sdf = 1e5;
    
    for (int i = 0; i < MAX_BUBBLES; i++) { 
        if (i >= u_bubble_count) break; 
        
        vec4 position_radius = read_position_radius(i);
        vec2 bubble_pos = position_radius.xy;
        float radius = position_radius.z;

        float d = length(world_pos - bubble_pos) - radius;
        outer_sdf = smin(outer_sdf, d, 35.0);
        
        if (d < d1) {
            d3 = d2;
            d2 = d1;
            d1 = d;
        } else if (d < d2) {
            d3 = d2;
            d2 = d;
        } else if (d < d3) {
            d3 = d;
        }
    }
    
    // --- PASS 2: Absolute distance weighting for smooth properties ---
    vec3 lab_accum = vec3(0.0);
    float light_accum = 0.0;
    float radius_accum = 0.0;
    float weight_sum = 0.0001;
    
    const float BLEED_RADIUS = 90.0;
    
    for (int i = 0; i < MAX_BUBBLES; i++) {
        if (i >= u_bubble_count) break;
        
        vec4 position_radius = read_position_radius(i);
        vec4 color_pressure = read_color_pressure(i);
        vec2 bubble_pos = position_radius.xy;
        float radius = position_radius.z;
        float light = position_radius.w;
        vec3 color = color_pressure.rgb;
        
        float d = length(world_pos - bubble_pos) - radius;
        float w = exp(-max(0.0, d) / BLEED_RADIUS);
        vec3 lab_col = rgb_to_oklab(srgb_to_linear(color));
        
        lab_accum += lab_col * w;
        light_accum += light * w;
        radius_accum += radius * w;
        weight_sum += w;
    }
    
    vec3 avg_lab = lab_accum / weight_sum;
    float avg_light = clamp(light_accum / (weight_sum), 0.0, 1000.0); // Cap maximum brightness
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
    
    // Supress the junction rounding effect when deep inside a cell to prevent ghost rays
    float junction_curve = wall_junction_blend + 8.0 + smoothstep(0.0, 15.0, raw_wall) * 100.0;
    float internal_wall = smin(raw_wall, junction_curve, 15.0);
    float internal_membrane = smoothstep(dynamic_wall_width, 0.0, internal_wall);
    // Gate the internal walls strictly by the smoothed cluster perimeter instead of d1.
    // This allows the wall to hit the bulging outer membrane perfectly without leaving a gap.
    float near_cluster = smoothstep(dynamic_wall_width, 0.0, outer_sdf);
    internal_membrane *= near_cluster;
    float outer_membrane = smoothstep(dynamic_wall_width, 0.0, abs(outer_sdf));
    float total_membrane = max(outer_membrane, internal_membrane);
    float inside_mask = smoothstep(1.5, -1.5, outer_sdf);
    
    // --- SHADING & COLORING ---
    vec3 background = vec3(0.015, 0.02, 0.04);
    const float DEPTH_FALLOFF = 45.0;
    
    float edge_dist = max(0.0, -outer_sdf);
    float cell_depth = 0.25 + 0.75 * exp(-edge_dist / DEPTH_FALLOFF);
    vec3 interior_color = blended_srgb * (0.2 + 0.8 * avg_light) * cell_depth;
    vec3 wall_color = vec3(0.95, 0.98, 1.0) * (0.25 + 0.75 * avg_light);
    vec3 final_color = mix(background, interior_color, inside_mask);
    final_color = mix(final_color, wall_color, total_membrane * 0.95);
    
    // --- HALO / GLOW ---
    const float HALO_BOOST = 1.4;
    // Scale halo spread based on average bubble radius (60.0 is the baseline reference size).
    // Clamped between 0.2 (tight min halo) and 3.0 (wide max halo).
    float halo_scale = clamp(avg_radius / 120.0, 0.2, 3.0);
    float dist = max(0.0, outer_sdf);
    float inner_halo = exp(-dist * (0.02 / halo_scale)) * 0.6;
    float outer_halo = exp(-dist * (0.005 / halo_scale)) * 0.4;
    float total_halo = (inner_halo + outer_halo) * avg_light * HALO_BOOST;
    
    final_color += blended_srgb * total_halo; fragColor = vec4(final_color, 1.0);
}