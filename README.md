# wgpu-mc

## 🚀 A blazing fast alternative renderer for Minecraft

#### Intro

WebGPU is a new web specification designed to provide modern graphics and compute capabilities in an API.
It is in its very early stages in web browsers, but has had a very promising cycle of development. It's inspired by
Metal & Vulkan's render pipelines, and is able to efficiently provide lower level access to graphics hardware, in a modern
and (relatively) easy to use API. 

`wgpu` is the name of a crate which implements this specification, and it is written in Rust, allowing safe and blazing-fast
use of the WebGPU standard, which makes it a prime candidate for a replacement of Blaze3D.

#### Usage

wgpu-mc is eventually meant to be a full replacement to the standard, official renderer "Blaze3D".
It will be used as a Fabric mod, which will disable the original OpenGL code and interface with wgpu-mc, using the native
Java interface.

#### Roadmap

World rendering

- [x] Discover and load blockmodels
- [x] Generate a texture atlas of all the block textures 
- [x] Convert the blockmodels into a mesh (not currently 100% accurate)
- [ ] Properly assign UVs to the meshes
- [ ] Generate optimized render chunk meshes

Gameplay

- [ ] Convert main menu and options menu into compatible code
- [ ] Render the in-game HUD
- [ ] Render the in-game chat
- [ ] Handle block animations

Entity rendering

- [ ] Render entity models
- [ ] Properly render entity parts and their animations

Java Interface

- [ ] Interface with the Fabric mod to use wgpu-mc

Shaders

- [ ] Have built in shaders that mimic the original Minecraft style, and also built-in more advanced shaders
- [ ] Ability to use custom shaders