precision highp float;

uniform sampler2D u_image;

uniform int u_mode;
uniform float u_now;
uniform vec2 u_resolution;

varying vec2 v_texCoord;

vec2 barrelDistortion(vec2 coord) {
    vec2 cc = coord - 0.5;
    float dist = dot(cc, cc);
    return coord + cc * dist * 0.3;
}

void main() {
    if(u_mode == 1) {
        /* RenderMode::Crt */

        vec2 uv = barrelDistortion(v_texCoord);

        // chromatic aberration offsets
        float aberration = 0.003;
        float r = texture2D(u_image, uv + vec2(aberration, 0.0)).r;
        float g = texture2D(u_image, uv).g;
        float b = texture2D(u_image, uv - vec2(aberration, 0.0)).b;

        vec3 color = vec3(r, g, b);

        // scanlines
        float scanline = sin(uv.y * u_resolution.y * 1.5);
        color *= 0.9 + 0.1 * scanline;

        // flicker
        float flicker = 0.95 + 0.05 * u_now;
        color *= flicker;

        // vignette
        vec2 center = uv - 0.5;
        color *= 1.0 - 0.2 * dot(center, center);

        gl_FragColor = vec4(color, 1.0);
    } else {
        /* RenderMode::None */
        gl_FragColor = texture2D(u_image, v_texCoord);
    }
}