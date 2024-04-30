#[repr(u8)]
pub enum PCICapabilityType {
    CommonConfig = 1,
    NotifyConfig = 2,
    ISRConfig = 3,
    DeviceConfig = 4,
    PCIConfig = 5,
    SharedMemoryConfig = 8,
    VendorConfig = 9,
}

#[repr(C)]
#[derive(Debug)]
pub struct CommonConfiguration {
    pub device_feature_select: u32,
    pub device_feature: u32,
    pub driver_feature_select: u32,
    pub driver_feature: u32,
    pub config_msix_vector: u16,
    pub num_queues: u16,
    pub device_status: u8,
    pub config_generation: u8,
    // About a specific virtqueue
    pub queue_select: u16,
    pub queue_size: u16,
    pub queue_msix_vector: u16,
    pub queue_enable: u16,
    pub queue_notify_off: u16,
    pub queue_desc: u64,
    pub queue_driver: u64,
    pub queue_device: u64,
    pub queue_notify_data: u16,
    pub queue_reset: u16,
}
