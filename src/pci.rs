use crate::{in32, out32};

pub struct DeviceIdentifier {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}
impl DeviceIdentifier {
    pub fn read_config(&self, dword_offset: u8) -> u32 {
        let addr = 0x8000_0000
            | ((self.bus as u32) << 16)
            | (((self.device as u32) & 0x1f) << 11)
            | (((self.function as u32) & 0x07) << 8)
            | (((dword_offset as u32) & 0x3f) << 2);

        unsafe {
            out32!(0xcf8, addr);
            in32!(0xcfc)
        }
    }

    pub fn read_device_vendor_ids(&self) -> [u16; 2] {
        unsafe { core::mem::transmute(self.read_config(0)) }
    }

    pub fn read_status_command_values(&self) -> [u16; 2] {
        unsafe { core::mem::transmute(self.read_config(1)) }
    }

    pub fn read_class_pif_revision_values(&self) -> [u8; 4] {
        unsafe { core::mem::transmute(self.read_config(2)) }
    }

    pub fn read_bist_ht_lt_cls_values(&self) -> [u8; 4] {
        unsafe { core::mem::transmute(self.read_config(3)) }
    }

    pub fn read_base_address_register(&self, n: u8) -> u32 {
        self.read_config(4 + n)
    }

    pub fn read_subsystem_ids(&self) -> [u16; 2] {
        unsafe { core::mem::transmute(self.read_config(11)) }
    }

    pub fn read_expansion_rom_base_address(&self) -> u32 {
        self.read_config(12)
    }

    pub fn read_capabilities_pointer(&self) -> u8 {
        (self.read_config(13) & 0xfc) as _
    }
}
