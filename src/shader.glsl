varying vec2 vUV;
varying vec4 vColor;

#ifdef VERTEX
attribute vec4 position;
attribute vec4 color;
attribute vec2 uv;

uniform mat4 transform;

void main() {
    vColor = color;
    vUV = uv;
    gl_Position = transform * position;
}
#endif

#ifdef FRAGMENT
uniform sampler2D font_sampler;

void main() {
    float alpha = Texel(font_sampler, vUV).a;

    if (alpha <= 0.0) {
        discard;
    }

    fragColor = vColor * alpha;
}
#endif