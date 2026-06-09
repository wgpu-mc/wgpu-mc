const TRI = array(
    vec4(-1.0, 3.0, 0.0, 1.0),
    vec4(3.0, -1.0, 0.0, 1.0),
    vec4(-1.0, -1.0, 0.0, 1.0)
);

struct Vert {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec4<f32>
}

struct Col {
    color: u32
}

var<immediate> colorIn: Col;

@fragment
fn frag(
    in: Vert
) -> @location(0) vec4<f32> {
    return in.col;
}

@vertex
fn vert(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) color: u32
) -> Vert {
    var v: Vert;

    v.pos = TRI[index];
    v.col = unpack4x8unorm(colorIn.color);

    return v;
}