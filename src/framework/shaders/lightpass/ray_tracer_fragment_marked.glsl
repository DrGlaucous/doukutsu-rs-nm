#version 110

//equivalent of "Texture" from the water shader
uniform sampler2D Texture; // The texture of the game's world/collision map.

uniform vec3 in_Light; // X, Y and Z (radius) of the light that is being ray-traced.

uniform vec2 in_World; // Size of the world/collision texture we're tracing rays against.
uniform float in_RayTexSize; // Size of the texture that the rays are being stored on.

//the pixels of the intermediate surface (the one you're not supposed to see)
//also Frag_UV
varying vec2 in_Coord; // The UV coordinate of the current pixel.

//hardcoded constants
const float MAXRADIUS = 65535.; // Maximum ray-length of 2 bytes, 2^16-1.
const float TAU = 6.2831853071795864769252867665590; // TAU or 2 * pi (shortcut for radial.circular math).

float map(float x, float in_min, float in_max, float out_min, float out_max) {
  return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}


void main() {
	// Converts the current pixel's coordinate from UV to XY space.
	vec2 Coord = floor(in_Coord * in_RayTexSize);
	
	// Takes the pixel's XY position, converts it to a vec2(1D-array index, ray count).
	//default "good" ray count is light radius * 2pi
	vec2 ray_count = vec2((Coord.y * in_RayTexSize) + Coord.x, TAU * in_Light.z);
	
	//cut unused indicies short (index >= ray count)
	//optimized out using "step" in the for loop (see below), but is in for now for clarification
	if (ray_count.x >= ray_count.y) {
		gl_FragColor = vec4(0,0,0,1);
		return;
	}
	
	// Takes the index/ray_count and converts it to an angle in range of: 0 to 2pi = 0 to ray_count.
	float Theta = TAU * (ray_count.x / ray_count.y);

	// get x/y coordinate from polar coordinate around light center
	vec2 Delta = vec2(cos(Theta), -sin(Theta));
	vec2 xyRay = vec2(0.0);

	// "Step" gets checks whether the current ray index < ray count, if not the ray is not traced (for-loop breaks).
	//step will be either 0 or 1, depending on if y is larger than x (if yes, we will do the loop. if not, MAXRADIUS is multiplied by 0 and we end early)
	for(float v = step(ray_count.x,ray_count.y), d = 0., radius = 0.; d < MAXRADIUS * v; d++) {
		/*
			"in_Light.z < d" Check if the current ray distance(length) "d" is > light radius (if so, then break).
			"d + in_Light.z * texture2D(...)" If collision in the world map at distance "d" is found, the ray ends
			(add light radius to d to make it greater than the light radius to break out of the for-loop.
		*/
		// if (in_Light.z < d + in_Light.z
		// * texture2D(Texture, (in_Light.xy + (xyRay = Delta * d)) * in_World).a) {
		// 	break;
		// }

		if (in_Light.z < d + in_Light.z
		* texture2D(Texture, (in_Light.xy + (xyRay = Delta * d)) * in_World).a) {
			break;
		}

		// if (radius < in_Light.z) {
		// 	xyRay = (Delta * d);
		// 	//if the point in the collision map has an alpha of 1, we'll add the light radius to "radius", breaking early
		// 	radius = d + (in_Light.z * texture2D(Texture, (in_Light.xy + xyRay) / in_World).a);
		// }


	}

	// Converts the ray length to polar UV coordinates ray_length / light_radius.
	//normalize ray length
	float rayLength = length(xyRay) / in_Light.z;
	

	//test
	//rayLength = map(Theta, 0., TAU, 0., 1.);
	if (Theta < TAU / 32.) {
		rayLength = 0.2;
	} else {
		rayLength = 0.9;
	}


	//red is MSB, blue is LSB
	// Takes the length of the current ray and splits it into two bytes and stores it in the texture.
	gl_FragColor = vec4(vec2(floor(rayLength * 255.0) / 255.0, fract(rayLength * 255.0)), 0.0, 1.0);

	//gl_FragColor = vec4(vec2(in_Coord.x), in_Coord.y, 1.0);//vec4(vec2(floor(rayLength * 255.0) / 255.0, fract(rayLength * 255.0)), 0.0, 1.0);

}
