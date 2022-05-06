# wgpu-mc

![img](media/logo.png)

## 🚀 A blazing fast alternative renderer for Minecraft
### Discord
https://discord.gg/NTuK8bQ2hn
### Matrix
https://matrix.to/#/#wgpu-mc:matrix.org

#### Intro

`wgpu` is a crate implementing the WebGPU specification in Rust. It's primary backends are Vulkan, DirectX 12, and Metal.

#### Goals

wgpu-mc is a standalone rendering engine for Minecraft-compatible projects. It's also a
replacement to Blaze3D using Fabric and the JNI to interface the two. 

#### Current status

The project is currently under active development. Quite a few important features have been implemented,
but not all of them. Feature parity with Blaze3D is the main goal at the moment, along with getting world rendering working
with Java Edition.

#### WIP and Completed Features

Engine

- [x] Block models from standard datapacks
- [x] Terrain rendering
- [x] Skybox support
- [x] Instanced Entity Rendering
- [ ] Animated textures
- [ ] Particles
- [ ] Lighting
- [ ] Item rendering

Minecraft

- [x] Disable Blaze3d
- [x] GUI rendering (text rendering is mangled but otherwise it works)
- [ ] World rendering
- [ ] Integrate entities
- [ ] Implement the Fabric Render API (Electrum)
- [ ] Particles
- [ ] Lighting
- [ ] Integrate Iris for more expansive/complex shader support

Pie in the sky

- [ ] Use https://github.com/birbe/jvm to run Minecraft in the browser