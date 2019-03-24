//! Vulkan version.

use log::{debug, info};
use vulkano::{
    device::{Device, DeviceExtensions, Features},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
};

pub fn main() {
    info!("Vulkan mode");

    // Create an instance of vulkan.
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("Failed to create vulkan instance");
    debug!("Successfully created vulkan instance: {:?}", instance);

    // List physical devices.
    for device in PhysicalDevice::enumerate(&instance) {
        debug!(
            "Physical device available [{}]: name={:?}, type={:?}, api_version={:?}",
            device.index(),
            device.name(),
            device.ty(),
            device.api_version()
        );
    }

    // Select a physical device.
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No physical devices available");
    info!(
        "Selected physical device: index={:?}, name={:?}, type={:?}, api_version={:?}",
        physical.index(),
        physical.name(),
        physical.ty(),
        physical.api_version()
    );

    // List device queue families.
    for family in physical.queue_families() {
        debug!(
            "Queue family found: id={:?}, count={:?}, \
             graphics={:?}, compute={:?}, transfers={:?}",
            family.id(),
            family.queues_count(),
            family.supports_graphics(),
            family.supports_compute(),
            family.supports_transfers()
        );
    }

    // Select a queue family.
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("No graphical queues available");
    info!(
        "Using queue family: id={:?}, count={:?}",
        queue_family.id(),
        queue_family.queues_count()
    );

    // Create device.
    let (_device, _queue) = {
        /// Queue priority, between 0.0 and 1.0.
        ///
        /// This can be any value in the range, because in this program only one
        /// queue family is used.
        const QUEUE_PRIORITY: f32 = 0.5;
        let (device, mut queues) = Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, QUEUE_PRIORITY)].iter().cloned(),
        )
        .expect("Failed to create device");
        (device, queues.next().expect("Should never fail"))
    };
    info!("Successfully created device object");
}
