pub trait Screen {
    fn render(&self, mouse_x: u32, mouse_y: u32, render_pass: &wgpu::RenderPass);
}
