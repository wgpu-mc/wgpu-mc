// use std::cell::{OnceCell, RefCell};
// use arrayvec::ArrayVec;
// use glam::{ivec2, ivec3, IVec3, Mat4};
// use parking_lot::lock_api::RwLock;
// use std::collections::HashMap;
// use std::fs;
// use std::path::PathBuf;
// use std::sync::Arc;
// use std::time::Instant;
// use winit::application::ApplicationHandler;
// use winit::dpi::PhysicalSize;
//
// use futures::executor::block_on;
// use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
// use winit::event_loop::EventLoop;
// use winit::keyboard::{KeyCode, PhysicalKey};
// use winit::window::Window;
// use crate::camera::Camera;
// use crate::chunk::make_chunks;
// use wgpu_mc::mc::direction::Direction;
// use wgpu_mc::mc::resource::{ResourcePath, ResourceProvider};
// use wgpu_mc::mc::Scene;
// use wgpu_mc::render::graph::{RenderGraph, ResourceBacking};
// use wgpu_mc::render::shaderpack::ShaderPackConfig;
// use wgpu_mc::wgpu::util::{BufferInitDescriptor, DeviceExt};
// use wgpu_mc::wgpu::{BufferBindingType, Extent3d, PresentMode};
// use wgpu_mc::{wgpu, Display, Frustum, WmRenderer};
//
// mod camera;
// mod chunk;
//
// struct FsResourceProvider {
//     pub asset_root: PathBuf,
// }
//
// //ResourceProvider is what wm uses to fetch resources. This is a basic implementation that's just backed by the filesystem
// impl ResourceProvider for FsResourceProvider {
//     fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>> {
//         let real_path = self.asset_root.join(id.0.replace(':', "/"));
//
//         fs::read(real_path).ok()
//     }
// }

fn main() {
    let out = wgpu_mc_jni::preprocessing::shim_samplers(
//         r#"#version 440
// layout(set = 4, binding = 0) uniform texture2D Sampler0_wm_t2d;
// layout(set = 4, binding = 1) uniform sampler Sampler0_wm_sampler;
// layout(set = 5, binding = 0) uniform texture2D Sampler1_wm_t2d;
// layout(set = 5, binding = 1) uniform sampler Sampler1_wm_sampler;
// layout(location = 0) out vec4 fragColor;
// layout(location = 2) in float sphericalVertexDistance;
// layout(location = 0) in vec4 texProj0;
// layout(location = 1) in float cylindricalVertexDistance;
// layout(set = 2, binding = 0) layout(std140) uniform Fog {
//   vec4 FogColor;
//
//   float FogEnvironmentalStart;
//
//   float FogEnvironmentalEnd;
//
//   float FogRenderDistanceStart;
//
//   float FogRenderDistanceEnd;
//
//   float FogSkyEnd;
//
//   float FogCloudsEnd;
//
// };
// float linear_fog_value(float vertexDistance, float fogStart, float fogEnd) {
//   if (vertexDistance <= fogStart) {
//     {
//       return 0.;
//     }
//   } else if (vertexDistance >= fogEnd) {
//     {
//       return 1.;
//     }
//   }
//   return (vertexDistance - fogStart) / (fogEnd - fogStart);
// }
// float total_fog_value(float sphericalVertexDistance, float cylindricalVertexDistance, float environmentalStart, float environmantalEnd, float renderDistanceStart, float renderDistanceEnd) {
//   return max(linear_fog_value(sphericalVertexDistance, environmentalStart, environmantalEnd), linear_fog_value(cylindricalVertexDistance, renderDistanceStart, renderDistanceEnd));
// }
// vec4 apply_fog(vec4 inColor, float sphericalVertexDistance, float cylindricalVertexDistance, float environmentalStart, float environmantalEnd, float renderDistanceStart, float renderDistanceEnd, vec4 fogColor) {
//   float fogValue = total_fog_value(sphericalVertexDistance, cylindricalVertexDistance, environmentalStart, environmantalEnd, renderDistanceStart, renderDistanceEnd);
//   return vec4(mix(inColor.rgb, fogColor.rgb, fogValue * fogColor.a), inColor.a);
// }
// float fog_spherical_distance(vec3 pos) {
//   return length(pos);
// }
// float fog_cylindrical_distance(vec3 pos) {
//   float distXZ = length(pos.xz);
//   float distY = abs(pos.y);
//   return max(distXZ, distY);
// }
// mat2 mat2_rotate_z(float radians) {
//   return mat2(cos(radians), -sin(radians), sin(radians), cos(radians));
// }
// layout(set = 3, binding = 0) layout(std140) uniform Globals {
//   ivec3 CameraBlockPos;
//
//   vec3 CameraOffset;
//
//   vec2 ScreenSize;
//
//   float GlintAlpha;
//
//   float GameTime;
//
//   int MenuBlurRadius;
//
//   int UseRgss;
//
// };
// const vec3[] COLORS = vec3[](vec3(0.022087, 0.098399, 0.110818), vec3(0.011892, 0.095924, 0.089485), vec3(0.027636, 0.101689, 0.100326), vec3(0.046564, 0.109883, 0.114838), vec3(0.064901, 0.117696, 0.097189), vec3(0.063761, 0.086895, 0.123646), vec3(0.084817, 0.111994, 0.16638), vec3(0.097489, 0.15412, 0.091064), vec3(0.106152, 0.131144, 0.195191), vec3(0.097721, 0.110188, 0.187229), vec3(0.133516, 0.138278, 0.148582), vec3(0.070006, 0.243332, 0.235792), vec3(0.196766, 0.142899, 0.214696), vec3(0.047281, 0.315338, 0.32197), vec3(0.204675, 0.39001, 0.302066), vec3(0.080955, 0.314821, 0.661491));
// const mat4 SCALE_TRANSLATE = mat4(0.5, 0., 0., 0.25, 0., 0.5, 0., 0.25, 0., 0., 1., 0., 0., 0., 0., 1.);
// mat4 end_portal_layer(float layer) {
//   mat4 translate = mat4(1., 0., 0., 17. / layer, 0., 1., 0., (2. + layer / 1.5) * (GameTime * 1.5), 0., 0., 1., 0., 0., 0., 0., 1.);
//   mat2 rotate = mat2_rotate_z(radians((layer * layer * 4321. + layer * 9.) * 2.));
//   mat2 scale = mat2((4.5 - layer / 4.) * 2.);
//   return mat4(scale * rotate) * translate * SCALE_TRANSLATE;
// }
//
// void main() {
//
//   vec3 color = textureProj(sampler2D(Sampler0_wm_t2d, Sampler0_wm_sampler), texProj0).rgb * COLORS[0];
//
//   for (int i = 0; i < PORTAL_LAYERS; i++) {
//
//     color += textureProj(sampler2D(Sampler1_wm_t2d, Sampler1_wm_sampler), texProj0 * end_portal_layer(float(i + 1))).rgb * COLORS[i];
//   }
//   fragColor = apply_fog(vec4(color, 1.), sphericalVertexDistance, cylindricalVertexDistance, FogEnvironmentalStart, FogEnvironmentalEnd, FogRenderDistanceStart, FogRenderDistanceEnd, FogColor);
//
// }"#
r#"#version 440
const vec3[] COLORS = vec3[](vec3(0.022087, 0.098399, 0.110818), vec3(0.011892, 0.095924, 0.089485), vec3(0.027636, 0.101689, 0.100326), vec3(0.046564, 0.109883, 0.114838), vec3(0.064901, 0.117696, 0.097189), vec3(0.063761, 0.086895, 0.123646), vec3(0.084817, 0.111994, 0.16638), vec3(0.097489, 0.15412, 0.091064), vec3(0.106152, 0.131144, 0.195191), vec3(0.097721, 0.110188, 0.187229), vec3(0.133516, 0.138278, 0.148582), vec3(0.070006, 0.243332, 0.235792), vec3(0.196766, 0.142899, 0.214696), vec3(0.047281, 0.315338, 0.32197), vec3(0.204675, 0.39001, 0.302066), vec3(0.080955, 0.314821, 0.661491));
const mat4 SCALE_TRANSLATE = mat4(0.5, 0., 0., 0.25, 0., 0.5, 0., 0.25, 0., 0., 1., 0., 0., 0., 0., 1.);


void main() {

}"#
                                                        , Box::new(vec![]), false);
    println!("{out}");
}

//
// struct Application {
//     wm: Option<WmRenderer>,
//     forward: f32,
//     scene: Option<Scene>,
//     render_graph: Option<RenderGraph>,
//     camera: Option<Camera>,
//     last_frame: Instant,
//     window: OnceCell<Arc<Window>>
// }
// impl Application {
//     pub fn new() -> Self {
//         Application {
//             wm: None,
//             forward: 0.0,
//             scene: None,
//             render_graph: None,
//             camera: None,
//             last_frame: Instant::now(),
//             window: OnceCell::new(),
//         }
//     }
// }
// impl ApplicationHandler for Application {
//     fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
//         let title = "wgpu-mc test";
//
//         let window_attributes = winit::window::Window::default_attributes()
//             .with_title(title)
//             .with_inner_size(winit::dpi::Size::Physical(PhysicalSize {
//                 width: 1280,
//                 height: 720,
//             }));
//         let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
//
//         self.window.set(window.clone()).unwrap();
//
//         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::PRIMARY,
//             ..Default::default()
//         });
//
//         let surface = instance.create_surface(window.clone()).unwrap();
//         let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
//             power_preference: wgpu::PowerPreference::HighPerformance,
//             force_fallback_adapter: false,
//             compatible_surface: Some(&surface),
//         }))
//         .unwrap();
//
//         let required_limits = wgpu::Limits {
//             max_push_constant_size: 128,
//             max_bind_groups: 8,
//             max_storage_buffers_per_shader_stage: 10000,
//             ..Default::default()
//         };
//
//         let (device, queue) = block_on(adapter.request_device(
//             &wgpu::DeviceDescriptor {
//                 label: None,
//                 required_features: wgpu::Features::default()
//                     | wgpu::Features::DEPTH_CLIP_CONTROL
//                     | wgpu::Features::PUSH_CONSTANTS
//                     | wgpu::Features::MULTI_DRAW_INDIRECT,
//                 required_limits,
//                 memory_hints: wgpu::MemoryHints::Performance,
//             },
//             None, // Trace path
//         ))
//         .unwrap();
//
//         const VSYNC: bool = true;
//
//         let surface_caps = surface.get_capabilities(&adapter);
//         let surface_config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format: wgpu::TextureFormat::Bgra8Unorm,
//             width: window.inner_size().width,
//             height: window.inner_size().height,
//             present_mode: if VSYNC {
//                 PresentMode::AutoVsync
//             } else if surface_caps.present_modes.contains(&PresentMode::Immediate) {
//                 PresentMode::Immediate
//             } else {
//                 surface_caps.present_modes[0]
//             },
//
//             desired_maximum_frame_latency: 2,
//             alpha_mode: surface_caps.alpha_modes[0],
//             view_formats: vec![],
//         };
//
//         surface.configure(&device, &surface_config);
//
//         let display = Display {
//             surface,
//             adapter,
//             device,
//             queue,
//             instance,
//             config: RwLock::new(surface_config),
//         };
//
//         let rsp = Arc::new(FsResourceProvider {
//             asset_root: crate_root::root()
//                 .unwrap()
//                 .join("wgpu-mc-demo")
//                 .join("res")
//                 .join("assets"),
//         });
//
//         let _mc_root = crate_root::root()
//             .unwrap()
//             .join("wgpu-mc-demo")
//             .join("res")
//             .join("assets")
//             .join("minecraft");
//
//         let wm = WmRenderer::new(display, rsp);
//
//         let blockstates_path = _mc_root.join("blockstates");
//
//         let blocks = {
//             //Read all of the blockstates in the Minecraft datapack folder thingy
//             let blockstate_dir = fs::read_dir(blockstates_path).unwrap();
//             // let mut model_map = HashMap::new();
//             let _bm = wm.mc.block_manager.write();
//
//             blockstate_dir.map(|m| {
//                 let model = m.unwrap();
//                 (
//                     format!(
//                         "minecraft:{}",
//                         model.file_name().to_str().unwrap().replace(".json", "")
//                     ),
//                     format!(
//                         "minecraft:blockstates/{}",
//                         model.file_name().to_str().unwrap()
//                     )
//                     .into(),
//                 )
//             })
//         }
//         .collect::<Vec<_>>();
//
//         wm.init();
//
//         wm.mc.bake_blocks(&wm, blocks.iter().map(|(a, b)| (a, b)));
//
//         let pack = serde_yaml::from_str::<ShaderPackConfig>(
//             &wm.mc
//                 .resource_provider
//                 .get_string(&ResourcePath("wgpu_mc:graph.yaml".into()))
//                 .unwrap(),
//         );
//
//         let mat4_model_buffer = Arc::new(create_buffer(&wm, &[0; 64]));
//         let mat4_view_buffer = Arc::new(create_buffer(&wm, &[0; 64]));
//         let mat4_persp_buffer = Arc::new(create_buffer(&wm, &[0; 64]));
//
//         let resource_backings = [
//             (
//                 "@mat4_model".into(),
//                 ResourceBacking::Buffer(mat4_model_buffer.clone(), BufferBindingType::Uniform),
//             ),
//             (
//                 "@mat4_view".into(),
//                 ResourceBacking::Buffer(mat4_view_buffer.clone(), BufferBindingType::Uniform),
//             ),
//             (
//                 "@mat4_perspective".into(),
//                 ResourceBacking::Buffer(mat4_persp_buffer.clone(), BufferBindingType::Uniform),
//             ),
//         ]
//         .into_iter()
//         .collect::<HashMap<String, ResourceBacking>>();
//
//         self.render_graph = Some(RenderGraph::new(
//             &wm,
//             pack.unwrap(),
//             resource_backings,
//             None,
//             None,
//         ));
//
//         self.scene = Some(Scene::new(
//             &wm,
//             Extent3d {
//                 width: window.inner_size().width,
//                 height: window.inner_size().height,
//                 depth_or_array_layers: 1,
//             },
//         ));
//
//         {
//             for x in 0..5 {
//                 for y in 0..2 {
//                     for z in 0..5 {
//                         make_chunks(&wm, [x, y, z].into(), self.scene.as_ref().unwrap());
//                     }
//                 }
//             }
//         }
//
//         self.camera = Some(Camera::new(
//             window.inner_size().width as f32
//                 / window.inner_size().height as f32,
//         ));
//
//         self.wm = Some(wm);
//     }
//
//     fn device_event(
//         &mut self,
//         _event_loop: &winit::event_loop::ActiveEventLoop,
//         _device_id: winit::event::DeviceId,
//         event: DeviceEvent,
//     ) {
//         if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
//             let camera = self.camera.as_mut().unwrap();
//             camera.yaw += (dx / 100.0) as f32;
//             camera.pitch -= (dy / 100.0) as f32;
//         }
//     }
//
//     fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
//         let wm = self.wm.as_ref().unwrap();
//         self.window.get().unwrap().request_redraw()
//     }
//
//     fn window_event(
//         &mut self,
//         event_loop: &winit::event_loop::ActiveEventLoop,
//         window_id: winit::window::WindowId,
//         event: WindowEvent,
//     ) {
//         let wm = self.wm.as_ref().unwrap();
//         if window_id == self.window.get().unwrap().id() {
//             match event {
//                 WindowEvent::CloseRequested => event_loop.exit(),
//                 WindowEvent::KeyboardInput { event, .. } => match event {
//                     KeyEvent {
//                         state: ElementState::Pressed,
//                         physical_key: PhysicalKey::Code(KeyCode::Space),
//                         ..
//                     } => {
//                         //Update a block and re-generate the chunk mesh for testing
//
//                         //removed atm
//                     }
//                     KeyEvent {
//                         state: ElementState::Pressed,
//                         physical_key: PhysicalKey::Code(KeyCode::Escape),
//                         ..
//                     } => event_loop.exit(),
//                     KeyEvent {
//                         state: ElementState::Pressed,
//                         physical_key: PhysicalKey::Code(KeyCode::KeyW),
//                         ..
//                     } => {
//                         self.forward = 1.0;
//                     }
//                     KeyEvent {
//                         state: ElementState::Released,
//                         physical_key: PhysicalKey::Code(KeyCode::KeyW),
//                         ..
//                     } => {
//                         self.forward = 0.0;
//                     }
//                     KeyEvent {
//                         state: ElementState::Pressed,
//                         physical_key: PhysicalKey::Code(KeyCode::KeyS),
//                         ..
//                     } => {
//                         self.forward = -1.0;
//                     }
//                     KeyEvent {
//                         state: ElementState::Released,
//                         physical_key: PhysicalKey::Code(KeyCode::KeyS),
//                         ..
//                     } => {
//                         self.forward = 0.0;
//                     }
//                     _ => {}
//                 },
//                 WindowEvent::RedrawRequested => {
//                     let camera = self.camera.as_mut().unwrap();
//                     let wm = self.wm.as_ref().unwrap();
//                     let frame_time = Instant::now().duration_since(self.last_frame).as_secs_f32();
//                     self.last_frame = Instant::now();
//
//                     camera.position += camera.get_direction() * self.forward * 50.0 * frame_time;
//
//                     let perspective: [[f32; 4]; 4] =
//                         camera.build_perspective_matrix().to_cols_array_2d();
//                     let view: [[f32; 4]; 4] = camera.build_view_matrix().to_cols_array_2d();
//
//                     if let ResourceBacking::Buffer(buffer, _) =
//                         &self.render_graph.as_ref().unwrap().resources["@mat4_model"]
//                     {
//                         wm.gpu.queue.write_buffer(
//                             buffer,
//                             0,
//                             bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
//                         );
//                     }
//                     *self.scene.as_mut().unwrap().camera_section_pos.write() = ivec2(
//                         camera.position.x.floor() as i32 >> 4,
//                         camera.position.z.floor() as i32 >> 4,
//                     );
//
//                     if let ResourceBacking::Buffer(buffer, _) =
//                         &self.render_graph.as_ref().unwrap().resources["@mat4_perspective"]
//                     {
//                         wm.gpu.queue.write_buffer(
//                             buffer,
//                             0,
//                             bytemuck::cast_slice(&perspective),
//                         );
//                     }
//
//                     if let ResourceBacking::Buffer(buffer, _) =
//                         &self.render_graph.as_ref().unwrap().resources["@mat4_view"]
//                     {
//                         wm.gpu
//                             .queue
//                             .write_buffer(buffer, 0, bytemuck::cast_slice(&view));
//                     }
//
//                     let mut config_guard = wm.gpu.config.write();
//
//                     let surface_texture =
//                         wm.gpu
//                             .surface
//                             .get_current_texture()
//                             .unwrap_or_else(|_| {
//                                 //The surface is outdated, so we force an update. This can't be done on the window resize event for synchronization reasons.
//                                 let size = self.window.get().unwrap().inner_size();
//
//                                 config_guard.width = size.width;
//                                 config_guard.height = size.height;
//
//                                 wm.gpu
//                                     .surface
//                                     .configure(&wm.gpu.device, &config_guard);
//                                 wm.gpu.surface.get_current_texture().unwrap()
//                             });
//
//                     let view = surface_texture
//                         .texture
//                         .create_view(&wgpu::TextureViewDescriptor {
//                             label: None,
//                             format: Some(wgpu::TextureFormat::Bgra8Unorm),
//                             dimension: Some(wgpu::TextureViewDimension::D2),
//                             aspect: Default::default(),
//                             base_mip_level: 0,
//                             mip_level_count: None,
//                             base_array_layer: 0,
//                             array_layer_count: None,
//                         });
//
//                     wm.submit_chunk_updates(self.scene.as_ref().unwrap());
//
//                     let mut command_encoder = wm
//                         .gpu
//                         .device
//                         .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//
//                     let mut geometry = HashMap::new();
//
//                     let mvp = (camera.build_perspective_matrix() * camera.build_view_matrix())
//                         .to_cols_array_2d();
//
//                     self.render_graph.as_ref().unwrap().render(
//                         wm,
//                         &mut command_encoder,
//                         self.scene.as_ref().unwrap(),
//                         &view,
//                         [0.0; 3],
//                         &mut geometry,
//                         &Frustum::from_modelview_projection(mvp),
//                     );
//
//                     wm.gpu.queue.submit([command_encoder.finish()]);
//
//                     surface_texture.present();
//                 }
//                 _ => {}
//             }
//         }
//     }
// }
//
// fn main() {
//     let a = 1;
//     let b = 1;
//     let c = 0;
//
//     let vertex_biases = ivec3(
//         if a == 0 { -1 } else { 1 },
//         if b == 0 { -1 } else { 1 },
//         if c == 0 { -1 } else { 1 },
//     );
//
//     let dir_vec = Direction::Up.to_vec();
//
//     let axis = dir_vec - vertex_biases; //equivalent to -(vertex_biases - dir_vec)
//
//     let mut axes: ArrayVec<IVec3, 2> = ArrayVec::new_const();
//
//     if axis.x != 0 {
//         axes.push(ivec3(axis.x, 0, 0));
//     }
//
//     if axis.y != 0 {
//         axes.push(ivec3(0, axis.y, 0));
//     }
//
//     if axis.z != 0 {
//         axes.push(ivec3(0, 0, axis.z));
//     }
//
//     let p1 = vertex_biases;
//     let p2 = p1 + axes[0];
//     let p3 = p1 + axes[1];
//     // let p4 = dir_vec;
//
//     dbg!(p1, p2, p3);
//
//     _main();
// }
//
// fn _main() {
//     let event_loop = EventLoop::new().unwrap();
//     let mut application = Application::new();
//     event_loop.run_app(&mut application).unwrap();
// }
//
// pub struct TerrainLayer;
//
// fn create_buffer(wm: &WmRenderer, contents: &[u8]) -> wgpu::Buffer {
//     wm.gpu.device.create_buffer_init(&BufferInitDescriptor {
//         label: None,
//         contents,
//         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//     })
// }
