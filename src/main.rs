use std::{iter, fs};

use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod model;
mod texture;

use model::{Vertex};
use wgpu_mc::Renderer;
use wgpu_mc::mc::chunk::Chunk;
use wgpu_mc::mc::block::{BlockState, StaticBlock, BlockModel, BlockDirection};
use std::collections::HashMap;
use std::cell::RefCell;
use futures::executor::block_on;
use std::path::PathBuf;
use wgpu_mc::mc::resource::{ResourceProvider, ResourceType};
use wgpu_mc::mc::datapack::NamespacedId;

struct SimpleResourceProvider {
    pub asset_root: PathBuf
}

impl ResourceProvider for SimpleResourceProvider {

    fn get_bytes(&self, t: ResourceType, id: &NamespacedId) -> Vec<u8> {

        let paths: Vec<&str> = match id {
            NamespacedId::Resource(res) => {
                res.1.split("/").take(2).collect()
            },
            _ => unreachable!()
        };

        let path = *paths.first().unwrap();
        let resource = format!("{}.png", *paths.last().unwrap());

        match t {
            ResourceType::Texture => {
                let real_path = self.asset_root.join("minecraft").join("textures").join(path).join(resource);
                fs::read(real_path).unwrap()
            }
        }

    }

}

fn main() {
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let mut state = block_on(Renderer::new(&window));

    let mc_root = std::path::Path::new(env!("OUT_DIR")).join("res").join("assets").join("minecraft");

    let rsp = SimpleResourceProvider {
        asset_root: std::path::Path::new(env!("OUT_DIR")).join("res").join("assets")
    };

    state.mc.load_block_models(mc_root);

    println!("Loaded block model datapacks.");

    state.mc.generate_block_texture_atlas(&rsp, &state.device, &state.queue, &state.texture_bind_group_layout);

    println!("Generated block texture atlas.");

    state.mc.generate_blocks(&state.device, &rsp);

    println!("Generated blocks.");

    // state.mc.blocks.iter().for_each(|e| {
    //     println!("{} = {{ textures: {:?}, model: {:?} }}", e.0, e.1.get_textures(), e.1.get_model());
    // });

    begin_rendering(event_loop, window, state);
}

fn begin_rendering(mut event_loop: EventLoop<()>, mut window: Window, mut state: Renderer<'static>) {
    use futures::executor::block_on;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            &state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            &state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                &state.update();
                match &state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => *(&state.resize(state.size)),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                };
            }
            _ => {}
        }
    });
}