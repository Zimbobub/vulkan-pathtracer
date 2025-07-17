


use std::sync::Arc;

use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};


pub struct GPU {
    pub instace: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub queue_family_index: u32
}


impl GPU {
    pub fn new() -> Self {
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
                .position(|(_queue_family_index, queue_family_properties)| {
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


        return Self { instace: instance, device: device, queue: queue, queue_family_index: queue_family_index };
    }
}