use core::ffi::c_void;

pub type EfiHandle = *mut core::ffi::c_void;
pub type EfiStatus = usize;

#[repr(C)]
#[derive(PartialEq, Eq)]
pub struct EfiGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}
impl core::fmt::Display for EfiGuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

#[repr(C)]
pub struct EfiTableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    _reserved: u32,
}

#[repr(C)]
pub struct EfiSystemTable {
    pub header: EfiTableHeader,
    pub firmware_vendor: *mut u16,
    pub firmware_revision: u32,
    pub console_in_handle: EfiHandle,
    pub con_in: *mut EfiSimpleTextInputProtocol,
    pub console_out_handle: EfiHandle,
    pub con_out: *mut EfiSimpleTextOutputProtocol,
    pub standard_error_handle: EfiHandle,
    pub std_err: *mut EfiSimpleTextOutputProtocol,
    pub runtime_services: *mut EfiRuntimeServices,
    pub boot_services: *mut EfiBootServices,
    pub number_of_table_entries: usize,
    pub configuration_table: *mut EfiConfigurationTable,
}
impl EfiSystemTable {
    #[inline]
    pub const fn configuration_table_entries(&self) -> &[EfiConfigurationTable] {
        unsafe {
            core::slice::from_raw_parts(self.configuration_table, self.number_of_table_entries)
        }
    }
}

#[repr(C)]
pub struct EfiSimpleTextInputProtocol {}

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    pub reset: *mut c_void,
    pub output_string: extern "system" fn(this: *mut Self, string: *mut u16) -> usize,
}

#[repr(C)]
pub struct EfiRuntimeServices {}

#[repr(C)]
pub struct EfiBootServices {
    pub header: EfiTableHeader,
    // Task Priority Services
    pub raise_tpl: *const c_void,
    pub restore_tpl: *const c_void,
    // Memory Services
    pub allocate_pages: *const c_void,
    pub free_pages: *const c_void,
    pub get_memory_map: extern "system" fn(
        memory_map_size: *mut usize,
        memory_map: *mut EfiMemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> EfiStatus,
    pub allocate_pool: *const c_void,
    pub free_pool: *const c_void,
    // Event & Timer Services
    pub create_event: *const c_void,
    pub set_timer: *const c_void,
    pub wait_for_event: *const c_void,
    pub signal_event: *const c_void,
    pub close_event: *const c_void,
    pub check_event: *const c_void,
    // Protocol Handler Services
    pub install_protocol_interface: *const c_void,
    pub reinstall_protocol_interface: *const c_void,
    pub uninstall_protocol_interface: *const c_void,
    pub handle_protocol: *const c_void,
    _reserved: *const c_void,
    pub register_protocol_notify: *const c_void,
    pub locate_handle: *const c_void,
    pub locate_device_path: *const c_void,
    pub install_configuration_table: *const c_void,
    // Image Services
    pub load_image: *const c_void,
    pub start_image: *const c_void,
    pub exit: *const c_void,
    pub unload_image: *const c_void,
    pub exit_boot_services:
        extern "system" fn(image_handle: EfiHandle, map_key: usize) -> EfiStatus,
    // Miscellaneous Services
    pub get_next_monotonic_count: *const c_void,
    pub stall: *const c_void,
    pub set_watchdog_timer: *const c_void,
    // Driver Support Services
    pub connect_controller: *const c_void,
    pub disconnect_controller: *const c_void,
    // Open and Close Protocol Services
    pub open_protocol: *const c_void,
    pub close_protocol: *const c_void,
    pub open_protocol_information: *const c_void,
    // Library Services
    pub protocols_per_handle: *const c_void,
    pub locate_handle_buffer: *const c_void,
    pub locate_protocol: extern "system" fn(
        protocol: *const EfiGuid,
        registration: *const c_void,
        interface: *mut *mut c_void,
    ) -> EfiStatus,
    pub install_multiple_protocol_interfaces: *const c_void,
    pub uninstall_multiple_protocol_interfaces: *const c_void,
    // 32-bit CRC Services
    pub calculate_crc32: *const c_void,
    // Miscellaneous Services
    pub copy_mem: *const c_void,
    pub set_mem: *const c_void,
    pub create_event_ex: *const c_void,
}

#[repr(C)]
pub struct EfiConfigurationTable {
    pub vendor_guid: EfiGuid,
    pub vendor_table: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct EfiMemoryAttributeTable {
    pub version: u32,
    pub number_of_entries: u32,
    pub descriptor_size: u32,
    pub flags: u32,
    pub entries: [EfiMemoryDescriptor; 0],
}
impl EfiMemoryAttributeTable {
    pub const GUID: EfiGuid = EfiGuid {
        data1: 0xdcfa911d,
        data2: 0x26eb,
        data3: 0x469f,
        data4: [0xa2, 0x20, 0x38, 0xb7, 0xdc, 0x46, 0x12, 0x20],
    };
}

#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub r#type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub query_mode: extern "system" fn(
        this: *mut Self,
        mode_number: u32,
        size_of_info: *mut usize,
        info: *mut *mut EfiGraphicsOutputModeInformation,
    ) -> EfiStatus,
    pub set_mode: extern "system" fn(this: *mut Self, mode_number: u32) -> EfiStatus,
    pub blt: extern "system" fn(
        this: *mut Self,
        blt_buffer: *mut EfiGraphicsOutputBltPixel,
        blt_operation: EfiGraphicsOutputBltOperation,
        source_x: usize,
        source_y: usize,
        destination_x: usize,
        destination_y: usize,
        width: usize,
        height: usize,
        delta: usize,
    ) -> EfiStatus,
    mode: *mut EfiGraphicsOutputProtocolMode,
}
impl EfiGraphicsOutputProtocol {
    pub const GUID: EfiGuid = EfiGuid {
        data1: 0x9042a9de,
        data2: 0x23dc,
        data3: 0x4a38,
        data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
    };

    #[inline]
    pub fn query_mode(
        &mut self,
        mode_number: u32,
        size_of_info: &mut usize,
        info: &mut *mut EfiGraphicsOutputModeInformation,
    ) -> EfiStatus {
        (self.query_mode)(self, mode_number, size_of_info, info)
    }

    #[inline]
    pub fn set_mode(&mut self, mode_number: u32) -> EfiStatus {
        (self.set_mode)(self, mode_number)
    }

    #[inline]
    pub fn blt(
        &mut self,
        blt_buffer: &mut [EfiGraphicsOutputBltPixel],
        blt_operation: EfiGraphicsOutputBltOperation,
        source_x: usize,
        source_y: usize,
        destination_x: usize,
        destination_y: usize,
        width: usize,
        height: usize,
        delta: usize,
    ) -> EfiStatus {
        (self.blt)(
            self,
            blt_buffer.as_mut_ptr(),
            blt_operation,
            source_x,
            source_y,
            destination_x,
            destination_y,
            width,
            height,
            delta,
        )
    }

    #[inline]
    pub fn mode(&self) -> &EfiGraphicsOutputProtocolMode {
        unsafe { &*self.mode }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EfiPixelBitmask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum EfiGraphicsPixelFormat {
    RedGreenBlueReserved8BitPerColor,
    BlueGreenRedReserved8BitPerColor,
    BitMask,
    BltOnly,
    FormatMax,
}

#[repr(C)]
pub struct EfiGraphicsOutputModeInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: EfiGraphicsPixelFormat,
    pub pixel_information: EfiPixelBitmask,
    pub pixels_per_scan_line: u32,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocolMode {
    pub max_mode: u32,
    pub mode: u32,
    info: *const EfiGraphicsOutputModeInformation,
    pub size_of_info: usize,
    pub frame_buffer_base: u64,
    pub frame_buffer_size: usize,
}
impl EfiGraphicsOutputProtocolMode {
    #[inline]
    pub const fn info(&self) -> &EfiGraphicsOutputModeInformation {
        unsafe { &*self.info }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EfiGraphicsOutputBltPixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}

#[repr(C)]
pub enum EfiGraphicsOutputBltOperation {
    BltVideoFill,
    BltVideoToBltBuffer,
    BltBufferToVideo,
    BltVideoToVideo,
}
