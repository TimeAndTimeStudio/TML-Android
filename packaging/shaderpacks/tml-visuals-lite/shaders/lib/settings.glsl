// TML Visuals Lite: the options its screen lists.

#define SUN_STRENGTH 2.4 // [1.5 2.0 2.4 2.8 3.2 3.6]
#define SKY_BRIGHTNESS 1.0 // [0.6 0.8 1.0 1.2 1.4]
#define BLOCKLIGHT_STRENGTH 1.0 // [0.5 0.75 1.0 1.25 1.5]
#define EXPOSURE 0.8 // [0.5 0.6 0.7 0.8 0.9 1.0 1.2 1.4]
#define SATURATION 1.05 // [0.8 0.9 1.0 1.05 1.1 1.2]

// Shadows: the map's size and reach, and how much of it goes near the player.
#define SHADOWS
const int shadowMapResolution = 1024; // [512 1024 2048]
const float shadowDistance = 64.0; // [32.0 48.0 64.0 96.0 128.0]
const float sunPathRotation = -25.0; // [-40.0 -30.0 -25.0 -15.0 0.0 15.0 25.0]
const bool shadowHardwareFiltering = true;
#define SHADOW_DISTORTION 0.9 // [0.0 0.7 0.85 0.9 0.93]
#define SHADOW_FILTER 4 // [1 4]
#define SHADOW_SOFTNESS 1.2 // [0.5 0.8 1.2 1.6 2.4]

// The air and the lens: haze, the sun's rays through it, and the glow around bright things.
#define FOG_DENSITY 1.0 // [0.0 0.5 0.75 1.0 1.5 2.0 3.0]
#define GODRAYS
#define GODRAYS_STRENGTH 1.0 // [0.5 0.75 1.0 1.5 2.0]
#define GODRAYS_SAMPLES 10 // [6 10 16]
#define BLOOM
#define BLOOM_STRENGTH 0.08 // [0.03 0.05 0.08 0.12 0.16]
// Clouds: 0 draws vanilla's own cloud mesh (cheapest), 1 a single flat layer of the pack's,
// 2 two soft layers. The pack's layers are the costliest thing it does.
#define CLOUD_STYLE 0 // [0 1 2]
#define CLOUDS_AMOUNT 1.0 // [0.0 0.35 0.6 0.8 1.0]
#define CLOUD_COVERAGE 0.65 // [0.2 0.35 0.45 0.55 0.65 0.75 0.9]
#define CLOUD_SPEED 1.0 // [0.0 0.5 1.0 1.5 2.0]

// A torch or another glowing block in the hand lights around the player.
#define HAND_LIGHT
