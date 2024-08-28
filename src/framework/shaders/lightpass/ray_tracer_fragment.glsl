#version 110
//version: stock ray tracer with annotations and new angle functions

//equivalent of "Texture" from the water shader
uniform sampler2D Texture; // The texture of the game's world/collision map.

uniform vec3 in_Light; // X, Y and Z (radius) of the light that is being ray-traced.

//in_World: Size of the world/collision texture we're tracing rays against.
//in_Angle: min and max valid angle to cast the light (in radians)
uniform vec2 in_World, in_Angle;

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
	// if (ray_count.x >= ray_count.y) {
	// 	gl_FragColor = vec4(0,0,0,1);
	// 	return;
	// }
	
	// Takes the index/ray_count and converts it to an angle in range of: 0 to 2pi = 0 to ray_count.
	float Theta = TAU * (ray_count.x / ray_count.y);

	// get x/y coordinate from polar coordinate around light center
	vec2 Delta = vec2(cos(Theta), -sin(Theta));
	vec2 xyRay = vec2(0.0);


	//will be 1 if the min_angle is bigger than the max_angle
	float min_angle_smaller = step(in_Angle.x, in_Angle.y);

	//step(x,y):
	//y>=x

	//1 if:
	//step(in_Angle.x, Theta) //1 if theta is larger= than min_angle AND
	//step(Theta, in_Angle.y) //1 if theta is smaller than max_angle AND
	//(1.0 - min_angle_bigger) //1 if we're looking at a normal range
	float is_within_normal_angle_range = step(in_Angle.x, Theta) * step(Theta, in_Angle.y) * min_angle_smaller;

	//1 if:
	//step(in_Angle.y, Theta) //1 if theta is larger= than max_angle AND
	//step(Theta, in_Angle.x) //1 if theta is smaller than min_angle AND
	//min_angle_bigger //1 if we're looking at an inverted range (pointing to the right)
	float is_within_inverted_range = (step(in_Angle.x, Theta) + step(Theta, in_Angle.y)) * (1.0 - min_angle_smaller);

	//is index less than ray count and are we within angle range?
	float in_valid_angle = step(ray_count.x,ray_count.y) * (is_within_normal_angle_range + is_within_inverted_range);

	// Ensures we are within the angle range prescribed. If not, the ray is not traced (for-loop breaks).
	// in_valid_angle is 1 if yes. If not, MAXRADIUS is multiplied by 0 and we end early)
	for(float v = in_valid_angle, d = 0., radius = 0.; d < MAXRADIUS * v; d++) {
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
	//normalize ray length (or zero if invalid range)
	float rayLength = (length(xyRay) / in_Light.z) * in_valid_angle;
	

	//test
	//rayLength = map(Theta, 0., TAU, 0., 1.);
	// if (Theta < TAU / 32.) {
	// 	rayLength = 0.2;
	// } else {
	// 	rayLength = 0.9;
	// }
	


	//red is MSB, blue is LSB
	// Takes the length of the current ray and splits it into two bytes and stores it in the texture.
	gl_FragColor = vec4(vec2(floor(rayLength * 255.0) / 255.0, fract(rayLength * 255.0)), 0.0, 1.0);


}
