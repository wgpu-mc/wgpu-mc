use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use futures::executor::block_on;
use parking_lot::RwLock;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use wgpu_mc::mc::block::{BlockMeshVertex, BlockstateKey};
use wgpu_mc::mc::chunk::{LightLevel, RenderLayer};
use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
use wgpu_mc::render::graph::{CustomResource, ResourceInternal, ShaderGraph};
use wgpu_mc::render::pipeline::Vertex;
use wgpu_mc::render::shaderpack::{Mat4, Mat4ValueOrMult};
use wgpu_mc::util::BindableBuffer;
use wgpu_mc::wgpu::BufferUsages;
use wgpu_mc::{wgpu, HasWindowSize, WindowSize, WmRenderer};

use crate::camera::Camera;
use crate::chunk::make_chunks;

mod camera;
mod chunk;
mod entity;

struct FsResourceProvider {
    pub asset_root: PathBuf,
}

//ResourceProvider is what wm uses to fetch resources. This is a basic implementation that's just backed by the filesystem
impl ResourceProvider for FsResourceProvider {
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
        let real_path = self.asset_root.join(id.0.replace(':', "/"));

        fs::read(real_path).ok()
    }
}

struct WinitWindowWrapper {
    window: Window,
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

unsafe impl HasRawDisplayHandle for WinitWindowWrapper {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let title = "wgpu-mc test";
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let wrapper = WinitWindowWrapper { window };

    let rsp = Arc::new(FsResourceProvider {
        asset_root: crate_root::root()
            .unwrap()
            .join("wgpu-mc-demo")
            .join("res")
            .join("assets"),
    });

    let _mc_root = crate_root::root()
        .unwrap()
        .join("wgpu-mc-demo")
        .join("res")
        .join("assets")
        .join("minecraft");

    let wgpu_state = block_on(WmRenderer::init_wgpu(&wrapper, false));

    let wm = WmRenderer::new(wgpu_state, rsp);

    wm.init();

    let blockstates_path = _mc_root.join("blockstates");

    let blocks = {
        //Read all of the blockstates in the Minecraft datapack folder thingy
        let blockstate_dir = fs::read_dir(blockstates_path).unwrap();
        // let mut model_map = HashMap::new();
        let _bm = wm.mc.block_manager.write();

        blockstate_dir.map(|m| {
            let model = m.unwrap();
            (
                format!(
                    "minecraft:{}",
                    model.file_name().to_str().unwrap().replace(".json", "")
                ),
                format!(
                    "minecraft:blockstates/{}",
                    model.file_name().to_str().unwrap()
                )
                .into(),
            )
        })
    }
    .collect::<Vec<_>>();

    let now = Instant::now();

    wm.mc.bake_blocks(&wm, blocks.iter().map(|(a, b)| (a, b)));

    let end = Instant::now();

    let window = wrapper.window;

    begin_rendering(event_loop, window, wm);
}

pub struct TerrainLayer;

impl RenderLayer for TerrainLayer {
    fn filter(&self) -> fn(BlockstateKey) -> bool {
        |_| true
    }

    fn mapper(&self) -> fn(&BlockMeshVertex, f32, f32, f32, LightLevel, bool) -> Vertex {
        |vert, x, y, z, _light, dark| Vertex {
            position: [
                vert.position[0] + x,
                vert.position[1] + y,
                vert.position[2] + z,
            ],
            uv: vert.tex_coords,
            normal: [vert.normal[0], vert.normal[1], vert.normal[2]],
            color: u32::MAX,
            uv_offset: vert.animation_uv_offset,
            lightmap_coords: 0,
            dark,
        }
    }

    fn name(&self) -> &str {
        "all"
    }
}

fn begin_rendering(event_loop: EventLoop<()>, window: Window, wm: WmRenderer) {
    // let (_entity, instances) = describe_entity(&wm);

    // let mut instances_map = HashMap::new();
    //
    // instances_map.insert(ENTITY_NAME.into(), instances);

    // wm.mc.entity_models.write().push(entity.clone());

    wm.pipelines
        .load_full()
        .chunk_layers
        .store(Arc::new(vec![Box::new(TerrainLayer)]));

    let chunk = make_chunks(&wm);

    {
        wm.mc
            .chunks
            .loaded_chunks
            .write()
            .insert([0, 0], ArcSwap::new(Arc::new(chunk)));
    }

    let mut frame_start = Instant::now();

    let mut forward = 0.0;
    let mut _spin: f32 = 0.0;
    let mut _frame: u32 = 0;

    let pack = serde_yaml::from_str(
        &wm.mc
            .resource_provider
            .get_string(&ResourcePath("wgpu_mc:graph.yaml".into()))
            .unwrap(),
    )
    .unwrap();
    let mut resources = HashMap::new();

    let aspect = {
        let surface = wm.wgpu_state.surface.read();
        (surface.1.width as f32) / (surface.1.height as f32)
    };

    let mut camera = Camera::new(aspect);
    let projection_bindable = Arc::new(BindableBuffer::new(
        &wm,
        &[0u8; 64],
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        "matrix",
    ));
    let view_bindable = Arc::new(BindableBuffer::new(
        &wm,
        &[0u8; 64],
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        "matrix",
    ));
    let rotation_bindable = Arc::new(BindableBuffer::new(
        &wm,
        &[0u8; 64],
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        "matrix",
    ));

    let view_matrix = Arc::new(RwLock::new(camera.build_view_matrix()));
    let projection_matrix = Arc::new(RwLock::new(camera.build_perspective_matrix()));
    let rotation_matrix = Arc::new(RwLock::new(camera.build_rotation_matrix()));

    resources.insert(
        "wm_mat4_projection".into(),
        CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Mat4(
                Mat4ValueOrMult::Value {
                    value: [[0.0; 4]; 4],
                },
                projection_matrix.clone(),
                projection_bindable.clone(),
            )),
        },
    );

    resources.insert(
        "wm_mat4_view".into(),
        CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Mat4(
                Mat4ValueOrMult::Value {
                    value: [[0.0; 4]; 4],
                },
                view_matrix.clone(),
                view_bindable.clone(),
            )),
        },
    );

    resources.insert(
        "wm_mat4_rotation".into(),
        CustomResource {
            update: None,
            data: Arc::new(ResourceInternal::Mat4(
                Mat4ValueOrMult::Value {
                    value: [[0.0; 4]; 4],
                },
                rotation_matrix.clone(),
                rotation_bindable.clone(),
            )),
        },
    );

    let mut graph = ShaderGraph::new(pack, resources, HashMap::new());

    graph.init(&wm, None, None);

    let instances = HashMap::new();

    event_loop
        .run(move |event, target| {
            match event {
                Event::AboutToWait => window.request_redraw(),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::KeyboardInput { event, .. } => match event {
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Space),
                                ..
                            } => {
                                //Update a block and re-generate the chunk mesh for testing

                                //removed atm
                            }
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            } => target.exit(),
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::KeyW),
                                ..
                            } => {
                                forward = 1.0;
                            }
                            KeyEvent {
                                state: ElementState::Released,
                                physical_key: PhysicalKey::Code(KeyCode::KeyW),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::KeyS),
                                ..
                            } => {
                                forward = -1.0;
                            }
                            KeyEvent {
                                state: ElementState::Released,
                                physical_key: PhysicalKey::Code(KeyCode::KeyS),
                                ..
                            } => {
                                forward = 0.0;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            wm.resize(WindowSize {
                                width: physical_size.width,
                                height: physical_size.height,
                            });
                        }
                        WindowEvent::RedrawRequested => {
                            let frame_time =
                                Instant::now().duration_since(frame_start).as_secs_f32();

                            // let mut part_transforms = vec![PartTransform::identity(); entity.parts.len()];
                            // let lid_index = *entity.parts.get("lid").unwrap();
                            // part_transforms[lid_index] = PartTransform {
                            //     x: 0.0,
                            //     y: 0.0,
                            //     z: 0.0,
                            //     pivot_x: (1.0 / 16.0),
                            //     pivot_y: (10.0 / 16.0),
                            //     pivot_z: (1.0 / 16.0),
                            //     yaw: 0.0,
                            //     pitch: ((spin * 15.0).sin() * PI).to_degrees() * 0.25 - 45.0,
                            //     roll: 0.0,
                            //     scale_x: 1.0,
                            //     scale_y: 1.0,
                            //     scale_z: 1.0,
                            // };

                            // let gpu_instance = EntityInstances::new(
                            //     entity.clone(),
                            //     vec![EntityInstanceTransforms {
                            //         position: (0.0, 0.0, 0.0),
                            //         looking_yaw: 0.0,
                            //         uv_offset: (0.0, 0.0),
                            //         part_transforms,
                            //     }],
                            // );

                            // gpu_instance.upload(&wm);

                            // instances_map.insert(ENTITY_NAME.into(), gpu_instance);

                            camera.position += camera.get_direction() * forward * frame_time * 40.0;

                            {
                                *projection_matrix.write() = camera.build_perspective_matrix();
                                *view_matrix.write() = camera.build_view_matrix();
                                *rotation_matrix.write() = camera.build_rotation_matrix();
                            }

                            let proj_mat: Mat4 = camera.build_perspective_matrix().into();
                            let view_mat: Mat4 = camera.build_view_matrix().into();
                            let rot_mat: Mat4 = camera.build_rotation_matrix().into();

                            wm.wgpu_state.queue.write_buffer(
                                &projection_bindable.buffer,
                                0,
                                bytemuck::cast_slice(&proj_mat),
                            );

                            wm.wgpu_state.queue.write_buffer(
                                &view_bindable.buffer,
                                0,
                                bytemuck::cast_slice(&view_mat),
                            );

                            wm.wgpu_state.queue.write_buffer(
                                &rotation_bindable.buffer,
                                0,
                                bytemuck::cast_slice(&rot_mat),
                            );

                            let surface_state = wm.wgpu_state.surface.read();
                            let surface = surface_state.0.as_ref().unwrap();
                            let texture = surface.get_current_texture().unwrap();
                            let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                                label: None,
                                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                                dimension: Some(wgpu::TextureViewDimension::D2),
                                aspect: Default::default(),
                                base_mip_level: 0,
                                mip_level_count: None,
                                base_array_layer: 0,
                                array_layer_count: None,
                            });

                            let _ = wm.render(&graph, &view, &surface_state.1, &instances);

                            texture.present();

                            frame_start = Instant::now();
                        }
                        _ => {}
                    }
                }
                // Event::DeviceEvent { event: DeviceEvent::Added {..}, ..} => {}
                // Event::DeviceEvent { event: DeviceEvent::Removed {..}, ..} => {}
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    camera.yaw += (delta.0 / 100.0) as f32;
                    camera.pitch -= (delta.1 / 100.0) as f32;
                }
                _ => {}
            }
        })
        .unwrap();
}
