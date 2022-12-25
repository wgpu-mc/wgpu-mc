struct Vertex {
    attributes: array<f32, 9>
}

struct ChunkInfo {
    start: u32,
    len: u32,
    x: f32,
    z: f32
}

//i32 because this internally starts out as -1 because when it's incremented then the first index we get is 0
//@group(0) @binding(0)
//var<storage, read_write> vbo_out_index: atomic<i32>;

@group(0) @binding(0)
var<storage, read_write> vbo_in: array<Vertex>;

//How many elements are in each chunk. Each vec2 corresponds to a workgroup id
//First component is the beginning index into the vbo_in array of this chunk. Second component is how many elements there are
//Third and fourth are XZ coordinates (MC block space, not chunk coordinates) of the chunk
@group(1) @binding(0)
var<storage> indices: array<ChunkInfo>;

@compute @workgroup_size(64)
fn assemble(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32
) {
    var chunk_element_info = indices[workgroup_id.x];

    var slice_start: u32 = chunk_element_info.start;
    var element_count: u32 = chunk_element_info.len;

    //Should always do at least one polygon
    var elements_per_invocation: u32 = max(1, element_count / 64u);
    var working_slice_start: u32 = slice_start + (elements_per_invocation * local_invocation_index);
    var working_slice_end: u32 = working_slice_start + elements_per_invocation;

    //If this is the last worker, then also make the worker do the remaining elements if the amount didn't cleanly divide
    if(local_invocation_index == 63) {
        working_slice_end += element_count % 64u;
    }

    if(!(element_count < 64 && local_invocation_index < element_count)) {
        for(var current_vertex_index = working_slice_start; current_vertex_index < working_slice_end; current_vertex_index = current_vertex_index + 1u) {
            vbo_in[current_vertex_index].attributes[0] += chunk_element_info.x;
            vbo_in[current_vertex_index].attributes[2] += chunk_element_info.z;
        }
    }
}