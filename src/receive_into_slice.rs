use bytemuck::AnyBitPattern;
use wgpu::{Buffer, BufferView, Device};

pub async fn receive_into_slice<T: AnyBitPattern>(
    device: &Device,
    buffer: Buffer,
    destination: &mut Vec<T>,
) {
    {
        let (tx, rx) = flume::bounded(1);
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            tx.send(result).unwrap()
        });
        device.poll(wgpu::PollType::Wait).unwrap();
        rx.recv_async().await.unwrap().unwrap();
        let output_data: BufferView = buffer.get_mapped_range(..);
        let out: &[T] = bytemuck::cast_slice(&output_data);
        destination.copy_from_slice(out);
    }
    buffer.unmap();
}
