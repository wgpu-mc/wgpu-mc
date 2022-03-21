use std::{fs};

use std::path::PathBuf;

use std::time::Instant;
use wgpu_mc::mc::datapack::{NamespacedResource};
use wgpu_mc::mc::block::{Block};

use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, DeviceEvent};
use wgpu_mc::{WmRenderer, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;

use std::sync::Arc;
use wgpu_mc::mc::resource::{ResourceProvider};
use std::convert::{TryFrom};

use wgpu_mc::render::pipeline::builtin::WorldPipeline;

use futures::StreamExt;
use std::collections::HashMap;


use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use fastanvil::{RegionBuffer};
use std::io::Cursor;
use fastanvil::pre18::JavaChunk;
use rayon::iter::{IntoParallelIterator};
use fastnbt::de::from_bytes;
use wgpu_mc::mc::entity::{EntityPart, PartTransform, Cuboid, CuboidUV, EntityManager};
use wgpu_mc::model::BindableTexture;
use wgpu_mc::render::shader::{WgslShader, WmShader};
use wgpu_mc::texture::TextureSamplerView;

struct SimpleResourceProvider {
    pub asset_root: PathBuf
}

impl ResourceProvider for SimpleResourceProvider {

    fn get_resource(&self, id: &NamespacedResource) -> Vec<u8> {
        let real_path = self.asset_root.join(&id.0).join(&id.1);
        fs::read(&real_path).unwrap_or_else(|_| { panic!("{}", real_path.to_str().unwrap().to_string()) })
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

fn load_anvil_chunks() -> Vec<(usize, usize, JavaChunk)> {
    let root = crate_root::root().unwrap().join("wgpu-mc-demo").join("res");
    let demo_root = root.join("demo_world");
    let region_dir = std::fs::read_dir(
        demo_root.join("region")
    ).unwrap();
    // let mut model_map = HashMap::new();

    let _begin = Instant::now();

    let regions: Vec<Vec<u8>> = region_dir.map(|region| {
        let region = region.unwrap();
        fs::read(region.path()).unwrap()
    }).collect();

    use rayon::iter::ParallelIterator;
    regions
        .into_par_iter()
        .flat_map(|region| {
            let cursor = Cursor::new(region);
            let mut region = RegionBuffer::new(cursor);
            let mut chunks = Vec::new();
            region.for_each_chunk(|x, z, chunk_data| {
                let chunk: JavaChunk = from_bytes(&chunk_data[..]).unwrap();
                chunks.push((x, z, chunk));
            });
            chunks
        }).collect()
}

fn main() {
    let anvil_chunks = load_anvil_chunks();

    let event_loop = EventLoop::new();
    let title = "wgpu-mc test";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let wrapper = WinitWindowWrapper {
        window
    };

    let rsp = Arc::new(SimpleResourceProvider {
        asset_root: crate_root::root().unwrap().join("wgpu-mc-demo").join("res").join("assets"),
    });

    let mc_root = crate_root::root()
        .unwrap()
        .join("wgpu-mc-demo")
        .join("res")
        .join("assets")
        .join("minecraft");

    let wgpu_state = block_on(WmRenderer::init_wgpu(&wrapper));

    let mut shaders = HashMap::new();

    for name in [
        "grass",
        "sky",
        "terrain",
        "transparent",
        "entity"
    ] {
        let wgsl_shader = WgslShader::init(
            &NamespacedResource::try_from("wgpu_mc:shaders/").unwrap().append(name).append(".wgsl"),
            &*rsp,
            &wgpu_state.device,
            "fs_main".into(),
            "vs_main".into()
        );

        shaders.insert(name.to_string(), Box::new(wgsl_shader) as Box<dyn WmShader>);
    }

    let wm = WmRenderer::new(
        wgpu_state,
        rsp,
        &shaders
    );

    let blockstates_path = mc_root.join("blockstates");

    {
        let blockstate_dir = std::fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let mut bm = wm.mc.block_manager.write();

        blockstate_dir.for_each(|m| {
            let model = m.unwrap();

            let resource_name = NamespacedResource (
                String::from("minecraft"),
                format!("blockstates/{}", model.file_name().to_str().unwrap())
            );

            match Block::from_json(model.file_name().to_str().unwrap(), std::str::from_utf8(&fs::read(model.path()).unwrap()).unwrap()) {
                None => {}
                Some(block) => { bm.blocks.insert(resource_name, block); }
            };
        });
    }

    println!("Generating blocks");
    wm.mc.bake_blocks(&wm);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm, anvil_chunks);
}

fn begin_rendering(event_loop: EventLoop<()>, window: Window, mut state: WmRenderer, chunks: Vec<(usize, usize, JavaChunk)>) {

    let _player_root = {
        EntityPart {
            transform: PartTransform {
                pivot_x: 0.0,
                pivot_y: 0.0,
                pivot_z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0
            },
            cuboids: vec![
                Cuboid {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    width: 1.0,
                    length: 1.0,
                    height: 1.0,
                    textures: CuboidUV {
                        north: ((0.0, 0.0), (0.0, 0.0)),
                        east: ((0.0, 0.0), (0.0, 0.0)),
                        south: ((0.0, 0.0), (0.0, 0.0)),
                        west: ((0.0, 0.0), (0.0, 0.0)),
                        up: ((0.0, 0.0), (0.0, 0.0)),
                        down: ((0.0, 0.0), (0.0, 0.0))
                    }
                }
            ],
            children: vec![]
        }
    };

    let alex_skin_ns: NamespacedResource = "minecraft:textures/entity/alex.png".try_into().unwrap();
    let alex_skin_resource = state.mc.resource_provider.get_resource(&alex_skin_ns);
    let alex_texture = BindableTexture::from_tsv(
        &*state.wgpu_state,
        &*state.pipelines.load_full(),
        TextureSamplerView::from_image_file_bytes(
            &*state.wgpu_state,
            &alex_skin_resource,
            "Alex"
        ).unwrap()
    );

    let entity_manager = EntityManager::new(
        &*state.wgpu_state,
        &state.pipelines.load_full()
    );



    let mut frame_start = Instant::now();
    let mut frame_time = 1.0;

    let mut forward = 0.0;

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        } => {
                            //Update a block and re-generate the chunk mesh for testing

                            //removed atm
                        },
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        },
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        } => {
                            forward = 1.0;
                        },
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        } => {
                            forward = 0.0;
                        },
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::S),
                            ..
                        } => {
                            forward = -1.0;
                        },
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::S),
                            ..
                        } => {
                            forward = 0.0;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        let _ = state.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let _ = state.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height
                        });
                    },
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let _ = state.update();

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                let mut camera = **state.mc.camera.load();

                let direction = camera.get_direction();
                camera.position += direction * 200.0 * frame_time * forward;

                state.mc.camera.store(Arc::new(camera));

                let _ = state.render(&[
                    &WorldPipeline {}
                ]);

                frame_start = Instant::now();
            },
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                match event {
                    // DeviceEvent::Added => {}
                    // DeviceEvent::Removed => {}
                    DeviceEvent::MouseMotion { delta } => {
                        let mut camera = **state.mc.camera.load();
                        camera.yaw += (delta.0 / 100.0) as f32;
                        camera.pitch -= (delta.1 / 100.0) as f32;
                        state.mc.camera.store(Arc::new(camera));
                    },
                    _ => {},
                }
            },
            _ => {}
        }
    });
}