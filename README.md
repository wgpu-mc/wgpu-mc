# wgpu-mc

## 🚀 A blazing fast alternative renderer for Minecraft
### Discord
https://discord.gg/NTuK8bQ2hn

#### Intro

WebGPU is a new web specification designed to provide modern graphics and compute capabilities in an API.
It is in its very early stages in web browsers, but has had a very promising cycle of development. It's inspired by
Metal & Vulkan's render pipelines, and is able to efficiently provide lower level access to graphics hardware, in a modern
and (relatively) easy to use API. 

`wgpu` is the name of a crate which implements this specification, and it is written in Rust, allowing safe and blazing-fast
use of the WebGPU standard, which makes it a prime candidate for a replacement of Blaze3D.

#### Goals

wgpu-mc is eventually meant to be a full replacement to the standard, official renderer "Blaze3D".
It will be used as a Fabric mod, which will disable the original OpenGL code and interface with wgpu-mc, using the native
Java interface.

#### Fabric

The repo for the Fabric mod which interfaces with wgpu-mc is located at https://github.com/birbe/wgpu-mc-fabric

#### Current status

The project is currently under active development (I'm solo at the moment though) and it's close
to getting a proper proof-of-concept working. The demo renderer works independently of the game, but does showcase
that the engine works. The main task is getting it to work with the game.

#### Roadmap

Engine

- [x] Load blockmodels
- [x] Generate a texture atlas of the textures 
- [x] Convert the block models into a mesh
- [x] Generate chunk meshes
- [ ] Skybox

Gameplay

- [x] Disable Blaze3d
- [ ] Upload basic chunk data to wgpu-mc

Entity rendering

- [ ] Render entity models
- [ ] Animations

Java Interface

- [x] Interface with the Fabric mod to use wgpu-mc

Shaders

- [ ] Have built in shaders that mimic the original Minecraft style, and also built-in more advanced shaders
- [ ] Ability to use custom shaders

Pie in the sky

- [ ] Use https://github.com/birbe/jvm to run Minecraft in the browser