#version 110

uniform sampler2D Texture;
varying vec2 Frag_UV; //uv corrdinate of current pixel
varying vec4 Frag_Color; //tint color

void main()
{
    gl_FragColor = Frag_Color * texture2D(Texture, Frag_UV.st);
}
