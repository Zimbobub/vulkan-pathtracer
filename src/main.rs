use std::sync::Arc;

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, descriptor_set::{DescriptorSet, WriteDescriptorSet}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo}, sync};
use vulkano::pipeline::Pipeline;

mod gpu;
mod shaders;

fn main() {
    let mut gpu = gpu::GPU::new();
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(gpu.device.clone()));




    ////////// Buffers

    let data_iter = 0..65536u32;
    let data_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data_iter,
    )
    .expect("failed to create buffer");



    //////////////// COMPUTE SETUP
    

    let shader = shaders::multiply_by_12::load(gpu.device.clone()).expect("Failed to create shader module");

    let cs = shader.entry_point("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(cs);
    let layout = PipelineLayout::new(
        gpu.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(gpu.device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline = ComputePipeline::new(
        gpu.device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline");





    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .unwrap();

        let descriptor_set = DescriptorSet::new(
            gpu.descriptor_set_allocator.clone(),
            descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding index
        [],
    )
    .unwrap();



    ///////////// Draw calls

    let mut builder = AutoCommandBufferBuilder::primary(
        gpu.command_buffer_allocator.clone(),
        gpu.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    ).unwrap();

    let work_group_counts = [1024, 1, 1];

    unsafe {
        builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                descriptor_set_layout_index as u32,
                descriptor_set,
            )
            .unwrap()
            .dispatch(work_group_counts)
            .unwrap();
    }

    let command_buffer = builder.build().unwrap();



    //////// Run & Results
    let start = std::time::Instant::now();
    gpu.run(command_buffer);
    println!("Done in {:.2?}", start.elapsed());

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }

    println!("Everything succeeded!");
}
