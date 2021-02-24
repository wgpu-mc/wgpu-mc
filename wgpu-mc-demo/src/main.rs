use std::{iter, fs};
use std::path::PathBuf;
use std::ops::DerefMut;
use std::time::Instant;
use wgpu_mc::mc::resource::{ResourceProvider, ResourceType};
use wgpu_mc::mc::datapack::NamespacedId;
use wgpu_mc::mc::block::{BlockDirection, BlockState};
use wgpu_mc::mc::chunk::{ChunkSection, Chunk};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};
use wgpu_mc::{Renderer, ShaderProvider, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

struct SimpleResourceProvider {
    pub asset_root: PathBuf
}

struct SimpleShaderProvider {
    pub shader_root: PathBuf
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

impl ShaderProvider for SimpleShaderProvider {
    fn get_shader(&self, name: &str) -> Vec<u8> {
        let path = self.shader_root.join(name);
        println!("{:?}", path);
        fs::read(path).unwrap()
    }
}

struct WinitWindowWrapper {
    window: Window
}

impl HasWindowSize for WinitWindowWrapper {
    fn get_window_size(&self) -> WindowSize {
        WindowSize {
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
        }
    }
}

unsafe impl HasRawWindowHandle for WinitWindowWrapper {

    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }

}

fn main() {
    let event_loop = EventLoop::new();
    let title = "wgpu-mc test";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let wrapper = WinitWindowWrapper {
        window
    };

    let sp = SimpleShaderProvider {
        shader_root: std::path::Path::new(env!("OUT_DIR")).join("res").join("shaders")
    };

    let rsp = SimpleResourceProvider {
        asset_root: std::path::Path::new(env!("OUT_DIR")).join("res").join("assets")
    };

    println!("{:?}", sp.shader_root);

    let mc_root = std::path::Path::new(env!("OUT_DIR")).join("res").join("assets").join("minecraft");

    println!("making renderer");

    let mut state = block_on(Renderer::new(&wrapper, Box::new(sp)));

    println!("doing stuff");

    state.mc.load_block_models(mc_root);

    println!("Loaded block model datapacks.");

    state.mc.generate_block_texture_atlas(&rsp, &state.device, &state.queue, &state.texture_bind_group_layout);

    println!("Generated block texture atlas.");

    state.mc.generate_blocks(&state.device, &rsp);

    let window = wrapper.window;

    begin_rendering(event_loop, window, state);
}

fn begin_rendering(mut event_loop: EventLoop<()>, mut window: Window, mut state: Renderer) {
    use futures::executor::block_on;

    let mut sections = Box::new([ChunkSection { empty: true, blocks: [BlockState {
        block: None,
        // block: None,
        direction: BlockDirection::North,
        damage: 0,
        is_cube: true
    }; 256] }; 256]);
    
    (0..5).for_each(|index| {
        sections.deref_mut()[index] = ChunkSection {
            empty: false,
            blocks: [BlockState {
                block: Option::Some(*state.mc.block_indices.get("minecraft:block/quartz_block").unwrap()),
                direction: BlockDirection::North,
                damage: 0,
                is_cube: true
            }; 256]
        };
    });

    let mut chunk = Chunk {
        pos: (0, 0),
        sections,
        vertices: None,
        vertex_buffer: None,
        vertex_count: 0
    };

    chunk.generate_vertices(&state.mc.blocks);
    chunk.upload_buffer(&state.device);

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
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            } => {
                                //Update a block and re-generate the chunk mesh for testing

                                println!("test");

                                chunk.sections.deref_mut()[3].blocks[0] = BlockState {
                                    block: Option::Some(*state.mc.block_indices.get("minecraft:block/quartz_block").unwrap()),
                                    direction: BlockDirection::North,
                                    damage: 0,
                                    is_cube: true
                                };

                                chunk.generate_vertices(&state.mc.blocks);
                                chunk.upload_buffer(&state.device);
                            }

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
                            &state.resize(WindowSize {
                                width: physical_size.width,
                                height: physical_size.height
                            });
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            &state.resize(WindowSize {
                                width: new_inner_size.width,
                                height: new_inner_size.height
                            });
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                &state.update();
                match &state.render_chunk(&chunk) {
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