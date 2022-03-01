pub trait WmShader {

    fn get_frag(&self) -> (&wgpu::ShaderModule, &str);

    fn get_vert(&self) -> (&wgpu::ShaderModule, &str);

}