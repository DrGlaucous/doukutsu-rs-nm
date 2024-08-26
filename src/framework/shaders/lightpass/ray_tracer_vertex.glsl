#version 110

uniform mat4 ProjMtx;
attribute vec2 Position;
attribute vec2 UV;
attribute vec4 Color;
varying vec2 in_Coord;
varying vec4 Frag_Color;

void main()
{
    in_Coord = UV; //Frag_UV
    Frag_Color = Color; //unused ATM
    gl_Position = ProjMtx * vec4(Position.xy, 0.0, 1.0);
}

// attribute vec3 in_Position;
// attribute vec2 in_TextureCoord;
// varying vec2 in_Coord;

// void main() {
//     gl_Position = gm_Matrices[MATRIX_WORLD_VIEW_PROJECTION] * vec4(in_Position, 1.0);
// 	in_Coord = in_TextureCoord;
// }
