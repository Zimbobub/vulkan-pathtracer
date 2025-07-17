use std::sync::Arc;

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, sync};
use vulkano::sync::GpuFuture;


mod gpu;

fn main() {
    let mut gpu = gpu::GPU::new();
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(gpu.device.clone()));



    let source_data: Vec<i32> = (0..64).collect();
    let source_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        source_data,
    )
    .expect("failed to create buffer");



    let dest_data: Vec<i32> = (0..64).map(|_| 0).collect();
    let dest_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        dest_data,
    )
    .expect("failed to create buffer");


    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        gpu.device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));



    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        gpu.queue_family_index,
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .copy_buffer(CopyBufferInfo::buffers(source_buffer.clone(), dest_buffer.clone())).unwrap();

    let command_buffer = builder.build().unwrap();


    let future = vulkano::sync::now(gpu.device.clone())
        .then_execute(gpu.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let src_content = source_buffer.read().unwrap();
    let destination_content = dest_buffer.read().unwrap();
    assert_eq!(&*src_content, &*destination_content);

    println!("Everything succeeded!");
}
