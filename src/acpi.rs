use crate::uefi::EfiGuid;

#[repr(C)]
#[derive(Debug)]
pub struct RootSystemDescriptionPointer {
    pub signature: u64,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    _reserved: [u8; 3],
}
impl RootSystemDescriptionPointer {
    pub const GUID_V2: EfiGuid = EfiGuid {
        data1: 0x8868e871,
        data2: 0xe4f1,
        data3: 0x11d3,
        data4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
    };

    pub fn has_correct_signature(&self) -> bool {
        self.signature == u64::from_le_bytes(*b"RSD PTR ")
    }

    #[inline]
    pub const unsafe fn xsdt(&self) -> &ExtendedSystemDescriptionTable {
        &*(self.xsdt_address as usize as *const ExtendedSystemDescriptionTable)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RootSystemDescriptionTable {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id_lo: u32,
    pub oem_table_id_hi: u32,
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
    entry: [u32; 0],
}
impl RootSystemDescriptionTable {
    pub fn has_correct_signature(&self) -> bool {
        self.signature == u32::from_le_bytes(*b"RSDT")
    }

    pub fn entries(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.entry.as_ptr(), (self.length as usize - 36) / 4) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ExtendedSystemDescriptionTable {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: u64,
    pub oem_revision_id: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
    // Note: offset=36に配置する必要があるのでアラインメントを4にしないといけない（u64だと8でずれる）
    entry: [[u32; 2]; 0],
}
impl ExtendedSystemDescriptionTable {
    pub fn has_correct_signature(&self) -> bool {
        self.signature == u32::from_le_bytes(*b"XSDT")
    }

    pub fn entries(&self) -> &[u64] {
        unsafe {
            core::slice::from_raw_parts(self.entry.as_ptr() as _, (self.length as usize - 36) / 8)
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SystemDescriptionTableHeader {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    // Note: for 4-byte alignment
    pub oem_table_id: [u32; 2],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}
impl SystemDescriptionTableHeader {
    #[inline]
    pub fn signature_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::mem::transmute::<_, &[u8; 4]>(&self.signature))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FixedDescriptionTable {
    pub header: SystemDescriptionTableHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    _reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_int: u16,
    pub smi_cmd: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_cnt: u8,
    pub pm1a_evt_blk: u32,
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32,
    pub pm1b_cnt_blk: u32,
    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32,
    pub gpe0_blk: u32,
    pub gpe1_blk: u32,
    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub mon_alarm: u8,
    pub century: u8,
    // Note: for 1-byte alignment
    pub iapc_boot_arch: [u8; 2],
    _reserved2: u8,
    pub flags: u32,
    pub reset_reg: [u8; 12],
    pub reset_value: u8,
    // Note: for 1-byte alignment
    pub arm_boot_arch: [u8; 2],
    pub fadt_minor_version: u8,
    // Note: for 4-byte alignment
    pub x_firmware_ctrl: [u32; 2],
    // Note: for 4-byte alignment
    pub x_dsdt: [u32; 2],
    pub x_pm1a_evt_blk: [u8; 12],
    pub x_pm1b_evt_blk: [u8; 12],
    pub x_pm1a_cnt_blk: [u8; 12],
    pub x_pm1b_cnt_blk: [u8; 12],
    pub x_pm2_cnt_blk: [u8; 12],
    pub x_pm_tmr_blk: [u8; 12],
    pub x_gpe0_blk: [u8; 12],
    pub x_gpe1_blk: [u8; 12],
    pub sleep_control_reg: [u8; 12],
    pub sleep_status_reg: [u8; 12],
    // Note: for 4-byte alignment
    pub hypervisor_vendor_identity: [u32; 2],
}
impl FixedDescriptionTable {
    pub const SIGNATURE: u32 = u32::from_ne_bytes(*b"FACP");
}

#[repr(C)]
pub struct MultipleAPICDescriptionTable {
    pub header: SystemDescriptionTableHeader,
    pub local_interrupt_controller_address: u32,
    pub flags: MultipleAPICDescriptionTableFlags,
    interrupt_controller_structure: [u8; 0],
}
impl MultipleAPICDescriptionTable {
    pub const SIGNATURE: u32 = u32::from_ne_bytes(*b"APIC");

    pub fn interrupt_controller_structure_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.interrupt_controller_structure.as_ptr(),
                self.header.length as usize - 44,
            )
        }
    }
}

#[repr(transparent)]
pub struct MultipleAPICDescriptionTableFlags(pub u32);
impl core::fmt::Debug for MultipleAPICDescriptionTableFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut v = self.0;
        let mut wrote = false;

        if (v & 0x01) != 0 {
            f.write_str(if wrote {
                " | PCAT_COMPAT"
            } else {
                "PCAT_COMPAT"
            })?;
            wrote = true;
            v &= !0x01;
        }

        if v != 0 {
            if wrote {
                write!(f, " | {v:08x}")?;
            } else {
                write!(f, "{v:08x}")?;
            }
        }

        Ok(())
    }
}

#[repr(C)]
pub struct ProcessorLocalAPICStructure {
    pub r#type: u8,
    pub length: u8,
    pub acpi_processor_uid: u8,
    pub apic_id: u8,
    pub flags: u32,
}
impl ProcessorLocalAPICStructure {
    pub const TYPE: u8 = 0x00;
}

#[repr(C)]
pub struct IOAPICStructure {
    pub r#type: u8,
    pub length: u8,
    pub io_apic_id: u8,
    _reserved: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}
impl IOAPICStructure {
    pub const TYPE: u8 = 0x01;
}

#[repr(C)]
pub struct InterruptSourceOverrideStructure {
    pub r#type: u8,
    pub length: u8,
    pub bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    pub flags: InterruptSourceOverrideFlags,
}
impl InterruptSourceOverrideStructure {
    pub const TYPE: u8 = 0x02;
}

#[repr(transparent)]
pub struct InterruptSourceOverrideFlags(pub u16);
impl InterruptSourceOverrideFlags {
    #[inline]
    pub const fn polarity(&self) -> u8 {
        (self.0 & 0x03) as _
    }

    #[inline]
    pub const fn trigger_mode(&self) -> u8 {
        ((self.0 >> 2) & 0x03) as _
    }
}
impl core::fmt::Debug for InterruptSourceOverrideFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let polarity = match self.polarity() {
            0 => "Conform",
            1 => "Active High",
            2 => "<Reserved>",
            3 => "Active Low",
            _ => unreachable!(),
        };
        let trigger_mode = match self.trigger_mode() {
            0 => "Conform",
            1 => "Edge",
            2 => "<Reserved>",
            3 => "Level",
            _ => unreachable!(),
        };
        let rest = self.0 >> 4;

        write!(f, "Polarity({polarity})|TriggerMode({trigger_mode})")?;
        if rest != 0 {
            write!(f, "|Rest({rest})")?;
        }

        Ok(())
    }
}

#[repr(C)]
pub struct LocalAPICNMIStructure {
    pub r#type: u8,
    pub length: u8,
    pub acpi_processor_uid: u8,
    pub flags: [u8; 2],
    pub local_apic_lint_number: u8,
}
impl LocalAPICNMIStructure {
    pub const TYPE: u8 = 0x04;
}
