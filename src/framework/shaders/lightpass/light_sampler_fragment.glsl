#version 110

//in_WorldMap: the light collision zone ("Texture" from the water shader)
//in_RayMap: holds the light ray info
uniform sampler2D Texture, RayTexture;

//in_Light: X, Y and Z (radius) of the light that is being ray-traced relative to the collision
//in_ColorS: color at center,
//in_ColorD: color at edge
uniform vec3 in_Light, in_ColorS, in_ColorD;

//in_WorldTexSize: size (w+h) of the texture collision texture (1/real_size)
//in_LightCenter: and the location of the light (relative to destination? ans: yes)
//in_LightTexSize_WH: size of the destination texture (any width+height)
uniform vec2 in_WorldTexSize, in_LightCenter, in_LightTexSize_WH;

//in_RayTexSize: the size of the raycast refrence texture
//in_LightTexSize: size of the destination texture (square?)
uniform float in_RayTexSize;//, in_LightTexSize;

//in_Coord: location of current pixel in uvspace
//also Frag_UV
varying vec2 in_Coord;


const float TAU = 6.2831853071795864769252867665590;
const float PI = TAU/2.;

// Custom tone map function, adjust as you please, keep in range 0 to 1.
float ToneMapFunc(float d, float m) {
	return clamp(1. - (d/m), 0., 1.);
}

void main() {
	// Gets the current pixel's texture XY coordinate from its texture UV coordinate.
	vec2 Coord = in_Coord * in_LightTexSize_WH;
	
	// Gets the lengthdir_xy of the current pixel in reference to the light position.
	//in_LightCenter is relative to where the pixel is (relative to target texture)
	vec2 Delta = Coord - in_LightCenter;

	// Gets the ray count as equal to the light's circumference.
	float RayCount = TAU * in_Light.z;
	

	// get index of the ray pointing toward this pixel
	float RayIndex = floor((RayCount * fract(atan(-Delta.y, Delta.x)/TAU)) + 0.5);

	//gets uv coordinates of the ray using rayIndex
	vec2 RayPos = vec2(mod(RayIndex, in_RayTexSize), RayIndex / in_RayTexSize) * (1./in_RayTexSize);


	//get ray's red and green values
	vec2 TexRay = texture2D(RayTexture, RayPos).rg;

	//get the distance from the current pixel to the light center.
	float Distance = distance(Coord, in_LightCenter);

	// turn RED and GREEN into a 16 bit integer (red is MSB, green is LSB)
	float RayLength = clamp(TexRay.r + (TexRay.g / 255.0), 0.0, 1.0) * in_Light.z;

	// Returns a bool whether or not this pixel is within the ray.
	//sign is + if pixel is within ray length
	//also check again if this pixel sits on a solid, so if the collision texture where the pixel is is opaque
	float RayVisible = max(RayLength - Distance, 0.0);// * (1. - texture2D(Texture, (in_Light.xy + Delta) * in_WorldTexSize).a);
	
	// Gets the gradient/tone map based on distance from the pixel to the light.
	float ToneMap = ToneMapFunc(Distance, in_Light.z);
	

	// Draw the final pixel output with the source and destination color lerp'd together, then apply the gradient/tonemap.
	//gl_FragColor = vec4(mix(in_ColorD, in_ColorS, vec3(ToneMap) * ToneMap), 1.0) * RayVisible;

	//interpolate colors using toneMap
	gl_FragColor = vec4(in_ColorS, 1.0) * RayVisible;
	//gl_FragColor = texture2D(Texture, vec2(0.0));
}

