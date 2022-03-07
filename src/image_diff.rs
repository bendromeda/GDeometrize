use wgpu::{ComputePassDescriptor, Texture};

use crate::{process::TOTAL_SHAPES, DiffBuffer, State};

pub fn calc_image_diff(
    state: &State,
    encoder: &mut wgpu::CommandEncoder,
    temp_view: &wgpu::TextureView,
) {
    state.queue.write_buffer(
        &state.diff_storage_buffer,
        0,
        // here is the tint
        bytemuck::cast_slice(&[DiffBuffer {
            diff: [0; TOTAL_SHAPES],
        }]),
    );

    let temp_texture_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &state.temp_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(temp_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&state.temp_texture.sampler),
            },
        ],
        label: Some("temp_texture_bind_group"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Image diff"),
        });
        compute_pass.set_pipeline(&state.compute_pipeline);

        compute_pass.set_bind_group(0, &state.target_bind_group, &[]);
        compute_pass.set_bind_group(1, &temp_texture_bind_group, &[]);
        compute_pass.set_bind_group(2, &state.size_bind_group, &[]);
        compute_pass.set_bind_group(3, &state.diff_bind_group, &[]);
        //compute_pass.set_bind_group(1, &state.current_bind_group, &[]);

        compute_pass.dispatch(
            state.size_uniform.width as u32,
            state.size_uniform.height as u32,
            TOTAL_SHAPES as u32,
        );
        // Number of cells to run, the (x,y,z) size of item being processed
    }
}

pub async fn get_image_diff(state: &State) -> [u32; TOTAL_SHAPES] {
    let buffer_slice = state.diff_storage_buffer.slice(..);

    let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
    state.device.poll(wgpu::Maintain::Wait);
    mapping.await.unwrap();

    let data: DiffBuffer = *bytemuck::from_bytes(&buffer_slice.get_mapped_range().to_vec());
    state.diff_storage_buffer.unmap();
    data.diff
}
