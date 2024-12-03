#version 330 core

in vec2 Frag_UV;
in vec4 Frag_Color;

out vec4 outColor;

void main()
{
    outColor = Frag_Color;
    //outColor = vec4(1.0, 0.2, 0.0, 1.0); // Red color
}
