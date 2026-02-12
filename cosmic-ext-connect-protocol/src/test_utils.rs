use crate::{Device, DeviceInfo, DeviceType};

pub fn create_test_device() -> Device {
    Device::from_discovery(DeviceInfo::new("Test Device", DeviceType::Desktop, 1716))
}
