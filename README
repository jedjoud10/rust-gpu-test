# Fully Rust Voxel DDA Implementation
This is a rewrite of [this project](https://github.com/jedjoud10/Voxel-Raymacher/tree/main) but instead of using C#, OpenGL and GLSL, this one uses Rust and WGPU only. The shaders themselves are written in rust and then compiled to SPIRV using [rust-gpu](https://github.com/Rust-GPU/rust-gpu)

Main Controls:
* W/A/S/D to move camera
* Hold left control to slow down camera speed
* Hold left shift to speed up camera speed (increase with scroll as well)
* Camera rotation using mouse
* F5 to toggle fullscreen
* Esc to close application

This implementation actually uses a proper DDA algorithm instead of whatever I was doing before (which was very slow). This new algorithm (that was kindly stolen from [here](https://www.shadertoy.com/view/lfyGRW) and with the help of ChatGPT to help explain it to me)

This algorithm is so much faster as a matter of fact, that I can render this at like 40fps at 1080p full screen with no hierarchical optimizations like the mip-level voxel octree or the BVH optimization that I had implemented before. With those implemented this could definitely get us at least 60fps on my current hardware.

Features of this implementation:
* Reflections like the old one
* Tinted refractions (with max refraction amount)
* Proper normals with no flickering due to better DDA algorithm

Features that I'd like to implement:
* Voxel Octree using mip-levels
* Sparse textures maybe if we switch to Vulkan? 
* BVH like in the OpenGL version
* Infinite chunk generation
* Celullar automata type of update (falling blocks, maybe some lighting effects?)
* Voxel Ambient Occlusion or SSAO
* Bloom, depth of field, tonemapping, PBR rendering, light shafts, volumetric lighting
* Voxel Global Illumination 