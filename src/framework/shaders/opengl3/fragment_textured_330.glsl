#version 330 core

uniform sampler2D Texture;
in vec2 Frag_UV;
in vec4 Frag_Color;

out vec4 outColor;

void main()
{
    outColor = Frag_Color * texture(Texture, Frag_UV.st);
    //outColor = vec4(1.0, 0.2, 0.0, 1.0); // Red color
}
