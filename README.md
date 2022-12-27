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

wgpu-mc is a standalone, mostly-batteries-included rendering engine written in Rust.
Electrum is a fabric mod that integrates wgpu-mc with Minecraft.

#### Current status

Both the engine and Electrum are both currently under active development.
wgpu-mc is fairly mature, while Electrum needs more development. The whole project is 
WIP, so something may work one day then be rewritten the next.
Terrain rendering works somewhat, while entities are still entirely un-integrated in Electrum.
A publicly testable release of the mod should be out Soonish.

#### WIP and Completed Features

Engine

- [x] Block models from standard datapacks
  - [x] Multipart
  - [x] Variants
- [x] Terrain rendering
- - [ ] Translucency sorting
- - [ ] Frustum culling compute shader
- [x] Skybox support
- [x] Instanced Entity Rendering
- [x] Animated textures
- [x] Particles
- [x] Advanced data-driven shader pipeline support

Minecraft

- [x] Disable Blaze3d
- [x] GUI rendering (text rendering is mangled but otherwise it works)
- [x] Terrain rendering
    - [ ] Lighting integration
    - [ ] Chunk updates
- [ ] Integrate entities
- [ ] Item rendering
- [ ] Implement the Fabric Rendering API (Electrum)
- [ ] Particles

Pie in the sky

- [ ] Use https://github.com/birbe/jvm to run Minecraft in the browser