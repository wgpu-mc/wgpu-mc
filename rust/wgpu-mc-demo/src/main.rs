use std::{fs};

use std::path::PathBuf;

use std::time::Instant;
use wgpu_mc::mc::datapack::{NamespacedResource};


use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, DeviceEvent};
use wgpu_mc::{WmRenderer, HasWindowSize, WindowSize};
use futures::executor::block_on;
use winit::window::Window;

use std::sync::Arc;
use wgpu_mc::mc::resource::{ResourceProvider};
use std::convert::{TryFrom};

use futures::StreamExt;
use std::collections::HashMap;


use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use fastanvil::{RegionBuffer};
use std::io::Cursor;

use fastanvil::pre18::JavaChunk;
use rayon::iter::{IntoParallelIterator};
use fastnbt::de::from_bytes;
use wgpu_mc::mc::entity::{EntityPart, PartTransform, Cuboid, CuboidUV, EntityManager, EntityModel, EntityInstance, DescribedEntityInstances};

use wgpu_mc::render::atlas::{Atlas, ATLAS_DIMENSIONS};
use wgpu_mc::render::entity::EntityRenderInstance;
use wgpu_mc::render::entity::pipeline::{EntityGroupInstancingFrame};
use wgpu_mc::render::pipeline::debug_lines::DebugLinesPipeline;
use wgpu_mc::render::pipeline::entity::EntityPipeline;
use wgpu_mc::render::pipeline::terrain::TerrainPipeline;
use wgpu_mc::render::pipeline::WmPipeline;
use wgpu_mc::render::shader::{WgslShader, WmShader};


use wgpu_mc::wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry};
use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};

struct FsResourceProvider {
    pub asset_root: PathBuf
}

impl ResourceProvider for FsResourceProvider {

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

    let rsp = Arc::new(FsResourceProvider {
        asset_root: crate_root::root().unwrap().join("wgpu-mc-demo").join("res").join("assets"),
    });

    let _mc_root = crate_root::root()
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
        rsp
    );

    wm.init(
        &[
            &EntityPipeline { frames: &[] },
            &TerrainPipeline,
            &DebugLinesPipeline
        ]
    );

    // let blockstates_path = mc_root.join("blockstates");
    //
    // {
    //     let blockstate_dir = std::fs::read_dir(blockstates_path).unwrap();
    //     // let mut model_map = HashMap::new();
    //     let mut bm = wm.mc.block_manager.write();
    //
    //     blockstate_dir.for_each(|m| {
    //         let model = m.unwrap();
    //
    //         let resource_name = NamespacedResource (
    //             String::from("minecraft"),
    //             format!("blockstates/{}", model.file_name().to_str().unwrap())
    //         );
    //
    //         match Block::from_json(model.file_name().to_str().unwrap(), std::str::from_utf8(&fs::read(model.path()).unwrap()).unwrap()) {
    //             None => {}
    //             Some(block) => { bm.blocks.insert(resource_name, block); }
    //         };
    //     });
    // }

    // println!("Generating blocks");
    // wm.mc.bake_blocks(&wm);

    let window = wrapper.window;

    println!("Starting rendering");
    begin_rendering(event_loop, window, wm, anvil_chunks);
}

fn begin_rendering(event_loop: EventLoop<()>, window: Window, wm: WmRenderer, _chunks: Vec<(usize, usize, JavaChunk)>) {

    let _atlas_1px = 1.0 / (ATLAS_DIMENSIONS as f32);
    let atlas_16px = 16.0 / (ATLAS_DIMENSIONS as f32);

    let _one = 1.0 / 16.0;

    let _player_root = {
        EntityPart {
            transform: PartTransform {
                pivot_x: 0.5,
                pivot_y: 0.5,
                pivot_z: 0.5,
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
                    height: 1.0,
                    length: 1.0,

                    textures: CuboidUV {
                        north: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                        east: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                        south: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                        west: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                        up: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                        down: ((0.0, 0.0), (atlas_16px, atlas_16px))
                    }
                }
            ],
            children: vec![
                //head
                // EntityPart {
                //     transform: PartTransform {
                //         pivot_x: 0.0,
                //         pivot_y: 0.0,
                //         pivot_z: 0.0,
                //         yaw: 0.0,
                //         pitch: 0.0,
                //         roll: 0.0
                //     },
                //     cuboids: vec![
                //         Cuboid {
                //             x: 0.0,
                //             y: one * 24.0,
                //             z: 0.0,
                //             width: one * 8.0,
                //             height: one * 10.0,
                //             length: one * 8.0,
                //             textures: CuboidUV {
                //                 north: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                //                 east: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                //                 south: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                //                 west: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                //                 up: ((0.0, 0.0), (atlas_16px, atlas_16px)),
                //                 down: ((0.0, 0.0), (atlas_16px, atlas_16px))
                //             }
                //         }
                //     ],
                //     children: vec![]
                // }
            ]
        }
    };

    let alex_skin_ns: NamespacedResource = "minecraft:textures/entity/alex.png".try_into().unwrap();
    let alex_skin_resource = wm.mc.resource_provider.get_resource(&alex_skin_ns);

    let player_atlas = Atlas::new(&*wm.wgpu_state, &*wm.render_pipeline_manager.load_full());

    player_atlas.allocate(
        &[
            (&alex_skin_ns, &alex_skin_resource)
        ]
    );

    player_atlas.upload(&wm);

    let entity_manager = EntityManager::new(
        &*wm.wgpu_state,
        &wm.render_pipeline_manager.load_full()
    );

    {
        *entity_manager.player_texture_atlas.write() = player_atlas;
    }

    let player_model = Arc::new(EntityModel {
        root: _player_root,
        parts: HashMap::new()
    });

    entity_manager.entity_types.write().push(
        player_model.clone()
    );

    let entity_instance = EntityInstance {
        entity_model: 0,
        position: (0.0, 0.0, 0.0),
        looking_yaw: 0.0,
        uv_offset: (0.0, 0.0),
        hurt: false,
        part_transforms: vec![
            PartTransform {
                pivot_x: 0.0,
                pivot_y: 0.0,
                pivot_z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0
            },
            // PartTransform {
            //     pivot_x: 0.0,
            //     pivot_y: 0.0,
            //     pivot_z: 0.0,
            //     yaw: 0.0,
            //     pitch: 0.0,
            //     roll: 0.0
            // }
        ]
    };

    let described_instance = entity_instance.describe_instance(
        &entity_manager
    );

    let (entity_instance_buffer, entity_instance_bind_group) = DescribedEntityInstances {
        matrices: vec![ described_instance ]
    }.upload(&wm);

    let entity_instance_buffer = Arc::new(entity_instance_buffer);
    let entity_instance_bind_group = Arc::new(entity_instance_bind_group);

    let entity_mesh_vertices = player_model.get_mesh();
    println!("{:?}", entity_mesh_vertices);

    let entity_vertex_buffer = Arc::new(wm.wgpu_state.device.create_buffer_init(
        &BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&entity_mesh_vertices),
            usage: wgpu_mc::wgpu::BufferUsages::VERTEX
        }
    ));

    let instance_buffer = Arc::new(wm.wgpu_state.device.create_buffer_init(
        &BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[EntityRenderInstance {
                entity_index: 0,
                entity_texture_index: 0,
                parts_per_entity: 1
            }]),
            usage: wgpu_mc::wgpu::BufferUsages::VERTEX
        }
    ));

    let texture_offsets_buffer = wm.wgpu_state.device.create_buffer_init(
        &BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&[
                0.0f32, 0.0f32
            ]),
            usage: wgpu_mc::wgpu::BufferUsages::STORAGE
        }
    );

    let texture_offsets_layout = wm.wgpu_state.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu_mc::wgpu::ShaderStages::VERTEX,
                ty: wgpu_mc::wgpu::BindingType::Buffer {
                    ty: wgpu_mc::wgpu::BufferBindingType::Storage {
                        read_only: true
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }
        ]
    });

    let texture_offsets_bind_group = Arc::new(wm.wgpu_state.device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &texture_offsets_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: texture_offsets_buffer.as_entire_binding()
            }
        ]
    }));

    let egif = Arc::new(EntityGroupInstancingFrame {
        vertex_buffer: entity_vertex_buffer,
        entity_instance_vb: instance_buffer,
        part_transform_matrices: entity_instance_bind_group,
        texture_offsets: texture_offsets_bind_group,
        texture: entity_manager.player_texture_atlas.read().bindable_texture.load_full(),
        instance_count: 1,
        vertex_count: entity_mesh_vertices.len() as u32
    });

    let mut frame_start = Instant::now();
    let mut frame_time = 1.0;

    let mut forward = 0.0;

    let mut spin = 0.0;

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
                        let _ = wm.resize(WindowSize {
                            width: physical_size.width,
                            height: physical_size.height
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let _ = wm.resize(WindowSize {
                            width: new_inner_size.width,
                            height: new_inner_size.height
                        });
                    },
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let _ = wm.update();

                frame_time = Instant::now().duration_since(frame_start).as_secs_f32();

                spin += 0.5;

                let entity_instance = EntityInstance {
                    entity_model: 0,
                    position: (0.0, 0.0, 0.0),
                    looking_yaw: 0.0,
                    uv_offset: (0.0, 0.0),
                    hurt: false,
                    part_transforms: vec![
                        PartTransform {
                            pivot_x: 0.0,
                            pivot_y: 0.0,
                            pivot_z: 0.0,
                            yaw: spin,
                            pitch: 0.0,
                            roll: 0.0
                        },
                        // PartTransform {
                        //     pivot_x: 0.0,
                        //     pivot_y: 0.0,
                        //     pivot_z: 0.0,
                        //     yaw: 0.0,
                        //     pitch: 0.0,
                        //     roll: 0.0
                        // },
                    ]
                };

                let described_instance = entity_instance.describe_instance(
                    &entity_manager
                );

                wm.wgpu_state.queue.write_buffer(
                    &*entity_instance_buffer.clone(),
                    0,
                    bytemuck::cast_slice(&described_instance)
                );

                let mut camera = **wm.mc.camera.load();

                let direction = camera.get_direction();
                camera.position += direction * 200.0 * frame_time * forward;

                wm.mc.camera.store(Arc::new(camera));

                let _ = wm.render(&[
                    // &WorldPipeline {}
                    &EntityPipeline {
                        frames: &[&*egif.clone()]
                    },
                    &DebugLinesPipeline
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
                        let mut camera = **wm.mc.camera.load();
                        camera.yaw += (delta.0 / 100.0) as f32;
                        camera.pitch -= (delta.1 / 100.0) as f32;
                        wm.mc.camera.store(Arc::new(camera));
                    },
                    _ => {},
                }
            },
            _ => {}
        }
    });
}