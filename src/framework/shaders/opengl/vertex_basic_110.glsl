#version 110

//the purpose of a vertex shader is to define varying variables for the fragment shader


//accessible at all stages
uniform mat4 ProjMtx;

///////////////////////////////////////////
//attribute keywords are only seen in the vertex shader

//corner position
attribute vec2 Position;

//normalized UV coordinate position (probably 0,0 or 1,1, or 1,0 or 0,1)
attribute vec2 UV;
//RGBA tint color
attribute vec4 Color;

///////////////////////////////////////////
//varying keywords are passed to the next stage directly

//normalized coordinate of current pixel
varying vec2 Frag_UV;

//RGBA tint
varying vec4 Frag_Color;
///////////////////////////////////////////

void main()
{
    Frag_UV = UV;
    Frag_Color = Color;
    gl_Position = ProjMtx * vec4(Position.xy, 0.0, 1.0);
}
