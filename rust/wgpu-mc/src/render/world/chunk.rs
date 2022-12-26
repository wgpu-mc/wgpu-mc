use std::sync::Arc;

use bytemuck::Pod;
use get_size::GetSize;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::mc::block::{
    BlockMeshVertex, BlockstateKey, ChunkBlockState, CubeOrComplexMesh, ModelMesh,
};
use crate::mc::chunk::{
    BlockStateProvider, Chunk, CHUNK_AREA, CHUNK_SECTION_HEIGHT, CHUNK_VOLUME, CHUNK_WIDTH,
};
use crate::mc::BlockManager;
use crate::WmRenderer;

fn get_block(block_manager: &BlockManager, state: ChunkBlockState) -> Option<Arc<ModelMesh>> {
    let key = match state {
        ChunkBlockState::Air => return None,
        ChunkBlockState::State(key) => key,
    };

    Some(
        block_manager
            .blocks
            .get_index(key.block as usize)?
            .1
            .get_model(key.augment),
    )
}

pub fn bake<
    T,
    Provider: BlockStateProvider,
    Filter: Fn(BlockstateKey) -> bool,
    Mapper: Fn(&BlockMeshVertex, f32, f32, f32) -> T,
>(
    block_manager: &BlockManager,
    chunk: &Chunk,
    mapper: Mapper,
    filter: Filter,
    state_provider: &Provider,
) -> Vec<T> {
    //Generates the mesh for this chunk, culling faces whenever possible
    let mut vertices = vec![];

    for block_index in 0..CHUNK_VOLUME {
        let x = (block_index % CHUNK_WIDTH) as i32;
        let y = (block_index / CHUNK_AREA) as i16;
        let z = ((block_index % CHUNK_AREA) / CHUNK_WIDTH) as i32;

        let absolute_x = (chunk.pos[0] * 16) + x;
        let absolute_z = (chunk.pos[1] * 16) + z;

        let _section_index = y / (CHUNK_SECTION_HEIGHT as i16);

        let block_state: ChunkBlockState = state_provider.get_state(absolute_x, y, absolute_z);

        if block_state.is_air() {
            continue;
        }

        let state_key = match block_state {
            ChunkBlockState::Air => unreachable!(),
            ChunkBlockState::State(key) => key,
        };

        if !filter(state_key) {
            continue;
        }

        let mesh = get_block(block_manager, block_state).unwrap();

        //TODO: randomly select a mesh if there are multiple

        match &mesh.models[0].0 {
            CubeOrComplexMesh::Cube(model) => {
                let render_north = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x, y, absolute_z - 1),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                let render_south = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x, y, absolute_z + 1),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                let render_up = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x, y + 1, absolute_z),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                let render_down = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x, y - 1, absolute_z),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                let render_west = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x - 1, y, absolute_z),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                let render_east = {
                    let state = get_block(
                        block_manager,
                        state_provider.get_state(absolute_x + 1, y, absolute_z),
                    );

                    match state {
                        Some(mesh) => mesh.models[0].1,
                        None => true,
                    }
                };

                if render_north {
                    match &model.north {
                        None => {}
                        Some(north) => vertices.extend(
                            north
                                .iter()
                                .map(|v| mapper(v, x as f32, y as f32, z as f32)),
                        ),
                    };
                }
                if render_east {
                    match &model.east {
                        None => {}
                        Some(east) => vertices
                            .extend(east.iter().map(|v| mapper(v, x as f32, y as f32, z as f32))),
                    };
                }
                if render_south {
                    match &model.south {
                        None => {}
                        Some(south) => vertices.extend(
                            south
                                .iter()
                                .map(|v| mapper(v, x as f32, y as f32, z as f32)),
                        ),
                    };
                }
                if render_west {
                    match &model.west {
                        None => {}
                        Some(west) => vertices
                            .extend(west.iter().map(|v| mapper(v, x as f32, y as f32, z as f32))),
                    };
                }
                if render_up {
                    match &model.up {
                        None => {}
                        Some(up) => vertices
                            .extend(up.iter().map(|v| mapper(v, x as f32, y as f32, z as f32))),
                    };
                }
                if render_down {
                    match &model.down {
                        None => {}
                        Some(down) => vertices
                            .extend(down.iter().map(|v| mapper(v, x as f32, y as f32, z as f32))),
                    };
                }
            }
            CubeOrComplexMesh::Complex(model) => {
                vertices.extend(
                    model
                        .iter()
                        .flat_map(|faces| {
                            [
                                faces.north.as_ref(),
                                faces.east.as_ref(),
                                faces.south.as_ref(),
                                faces.west.as_ref(),
                                faces.up.as_ref(),
                                faces.down.as_ref(),
                            ]
                        })
                        .flatten()
                        .flatten()
                        .map(|v| mapper(v, x as f32, y as f32, z as f32)),
                );
            }
        }
    }

    vertices
}
