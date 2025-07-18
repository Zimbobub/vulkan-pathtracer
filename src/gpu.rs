


use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::sync::GpuFuture;





pub struct GPU {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>
}


impl GPU {
    pub fn init() -> Self {
        // get vulkan instance
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("failed to create instance");


        // get devices
        let physical_devices: Vec<Arc<PhysicalDevice>> = instance.enumerate_physical_devices().expect("could not enumerate devices").collect();

        for physical_device in physical_devices.iter() {
            println!("Device: {}", physical_device.properties().device_name.clone());
        }

        let physical_device = physical_devices.iter().next().expect("no devices available");


        // get virtual device
        for family in physical_device.queue_family_properties() {
            println!("Found a queue family with {:?} queue(s). {:?}", family.queue_count, family.queue_flags);
        }

        // pick queue
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_, queue_family_properties)| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family") as u32;

        // create device
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("failed to create device");


        let queue = queues.next().unwrap();



        /////////////////// ALLOCATORS


        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default()
        ));



        return Self {
            device: device,
            queue:queue,
            memory_allocator: memory_allocator,
            command_buffer_allocator: command_buffer_allocator,
            descriptor_set_allocator: descriptor_set_allocator
        };
    }


    pub fn buffer_from_iter<I, T>(&self, data: I, usage: BufferUsage, memory_type_filter: MemoryTypeFilter) -> Subbuffer<[T]> where T: BufferContents, I: IntoIterator<Item = T>, I::IntoIter: ExactSizeIterator {
        return Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo { usage: usage, ..Default::default() },
            AllocationCreateInfo { memory_type_filter: memory_type_filter, ..Default::default() },
            data
        ).expect("failed to create buffer");
    }


    pub fn run(&self, command_buffer: Arc<PrimaryAutoCommandBuffer>) {
        let future = vulkano::sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }
}