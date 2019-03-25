//! Vulkan setup.

use std::sync::Arc;

use failure::{format_err, Fallible, ResultExt};
use log::{debug, info};
use vulkano::{
    device::{Device, DeviceExtensions, Features, Queue},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
};

/// Initialize vulkan.
pub fn setup() -> Fallible<(Arc<Device>, Arc<Queue>)> {
    // Create an instance of vulkan.
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .with_context(|e| format!("Failed to create vulkan instance: {}", e))?;
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
        .ok_or_else(|| format_err!("No physical devices available"))?;
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
        .ok_or_else(|| format_err!("No graphical queues available"))?;
    info!(
        "Using queue family: id={:?}, count={:?}",
        queue_family.id(),
        queue_family.queues_count()
    );

    // Create device.
    let (device, queue) = {
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
        .with_context(|e| format!("Failed to create device: {}", e))?;
        (device, queues.next().expect("Should never fail"))
    };
    info!("Successfully created device object");

    Ok((device, queue))
}
