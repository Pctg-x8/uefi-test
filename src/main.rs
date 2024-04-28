#![no_std]
#![no_main]

use core::{ffi::c_void, fmt::Write, panic::PanicInfo};

static mut SYSTEM_TABLE: *mut EfiSystemTable = core::ptr::null_mut();

#[panic_handler]
fn panic_handler<'a, 'b>(info: &'a PanicInfo<'b>) -> ! {
    let mut con_out = ConsoleWriter {
        protocol: unsafe { (*SYSTEM_TABLE).con_out },
    };

    writeln!(&mut con_out, "[PANIC OCCURED] {info}").unwrap();

    loop {}
}

type EfiHandle = *mut core::ffi::c_void;
type EfiStatus = usize;

#[repr(C)]
#[derive(PartialEq, Eq)]
struct EfiGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
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
struct EfiTableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    _reserved: u32,
}

#[repr(C)]
struct EfiSystemTable {
    header: EfiTableHeader,
    firmware_vendor: *mut u16,
    firmware_revision: u32,
    console_in_handle: EfiHandle,
    con_in: *mut EfiSimpleTextInputProtocol,
    console_out_handle: EfiHandle,
    con_out: *mut EfiSimpleTextOutputProtocol,
    standard_error_handle: EfiHandle,
    std_err: *mut EfiSimpleTextOutputProtocol,
    runtime_services: *mut EfiRuntimeServices,
    boot_services: *mut EfiBootServices,
    number_of_table_entries: usize,
    configuration_table: *mut EfiConfigurationTable,
}

#[repr(C)]
struct EfiSimpleTextInputProtocol {}

#[repr(C)]
struct EfiSimpleTextOutputProtocol {
    reset: *mut c_void,
    output_string:
        extern "system" fn(this: *mut EfiSimpleTextOutputProtocol, string: *mut u16) -> usize,
}

#[repr(C)]
struct EfiRuntimeServices {}

#[repr(C)]
struct EfiBootServices {
    header: EfiTableHeader,
    // Task Priority Services
    raise_tpl: *const c_void,
    restore_tpl: *const c_void,
    // Memory Services
    allocate_pages: *const c_void,
    free_pages: *const c_void,
    get_memory_map: *const c_void,
    allocate_pool: *const c_void,
    free_pool: *const c_void,
    // Event & Timer Services
    create_event: *const c_void,
    set_timer: *const c_void,
    wait_for_event: *const c_void,
    signal_event: *const c_void,
    close_event: *const c_void,
    check_event: *const c_void,
    // Protocol Handler Services
    install_protocol_interface: *const c_void,
    reinstall_protocol_interface: *const c_void,
    uninstall_protocol_interface: *const c_void,
    handle_protocol: *const c_void,
    _reserved: *const c_void,
    register_protocol_notify: *const c_void,
    locate_handle: *const c_void,
    locate_device_path: *const c_void,
    install_configuration_table: *const c_void,
    // Image Services
    load_image: *const c_void,
    start_image: *const c_void,
    exit: *const c_void,
    unload_image: *const c_void,
    exit_boot_services: *const c_void,
    // Miscellaneous Services
    get_next_monotonic_count: *const c_void,
    stall: *const c_void,
    set_watchdog_timer: *const c_void,
    // Driver Support Services
    connect_controller: *const c_void,
    disconnect_controller: *const c_void,
    // Open and Close Protocol Services
    open_protocol: *const c_void,
    close_protocol: *const c_void,
    open_protocol_information: *const c_void,
    // Library Services
    protocols_per_handle: *const c_void,
    locate_handle_buffer: *const c_void,
    locate_protocol: extern "system" fn(
        protocol: *const EfiGuid,
        registration: *const c_void,
        interface: *mut *mut c_void,
    ) -> EfiStatus,
    install_multiple_protocol_interfaces: *const c_void,
    uninstall_multiple_protocol_interfaces: *const c_void,
    // 32-bit CRC Services
    calculate_crc32: *const c_void,
    // Miscellaneous Services
    copy_mem: *const c_void,
    set_mem: *const c_void,
    create_event_ex: *const c_void,
}

#[repr(C)]
struct EfiConfigurationTable {
    vendor_guid: EfiGuid,
    vendor_table: *mut core::ffi::c_void,
}

#[repr(C)]
struct EfiMemoryAttributeTable {
    version: u32,
    number_of_entries: u32,
    descriptor_size: u32,
    flags: u32,
    entries: [EfiMemoryDescriptor; 0],
}
impl EfiMemoryAttributeTable {
    const GUID: EfiGuid = EfiGuid {
        data1: 0xdcfa911d,
        data2: 0x26eb,
        data3: 0x469f,
        data4: [0xa2, 0x20, 0x38, 0xb7, 0xdc, 0x46, 0x12, 0x20],
    };
}

#[repr(C)]
struct EfiMemoryDescriptor {
    r#type: u32,
    physical_start: u64,
    virtual_start: u64,
    number_of_pages: u64,
    attribute: u64,
}

#[repr(C)]
struct EfiGraphicsOutputProtocol {
    query_mode: extern "system" fn(
        this: *mut EfiGraphicsOutputProtocol,
        mode_number: u32,
        size_of_info: *mut usize,
        info: *mut *mut EfiGraphicsOutputModeInformation,
    ) -> EfiStatus,
    set_mode:
        extern "system" fn(this: *mut EfiGraphicsOutputProtocol, mode_number: u32) -> EfiStatus,
    blt: extern "system" fn(
        this: *mut EfiGraphicsOutputProtocol,
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
    const GUID: EfiGuid = EfiGuid {
        data1: 0x9042a9de,
        data2: 0x23dc,
        data3: 0x4a38,
        data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
    };
}

#[repr(C)]
#[derive(Debug)]
struct EfiPixelBitmask {
    red: u32,
    green: u32,
    blue: u32,
    reserved: u32,
}

#[repr(C)]
#[derive(Debug)]
enum EfiGraphicsPixelFormat {
    RedGreenBlueReserved8BitPerColor,
    BlueGreenRedReserved8BitPerColor,
    BitMask,
    BltOnly,
    FormatMax,
}

#[repr(C)]
struct EfiGraphicsOutputModeInformation {
    version: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: EfiGraphicsPixelFormat,
    pixel_information: EfiPixelBitmask,
    pixels_per_scan_line: u32,
}

#[repr(C)]
struct EfiGraphicsOutputProtocolMode {
    max_mode: u32,
    mode: u32,
    info: *const EfiGraphicsOutputModeInformation,
    size_of_info: usize,
    frame_buffer_base: u64,
    frame_buffer_size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EfiGraphicsOutputBltPixel {
    blue: u8,
    green: u8,
    red: u8,
    reserved: u8,
}

#[repr(C)]
enum EfiGraphicsOutputBltOperation {
    BltVideoFill,
    BltVideoToBltBuffer,
    BltBufferToVideo,
    BltVideoToVideo,
}

#[repr(C)]
#[derive(Debug)]
struct AcpiRootSystemDescriptionPointer {
    signature: u64,
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    _reserved: [u8; 3],
}
impl AcpiRootSystemDescriptionPointer {
    const GUID2: EfiGuid = EfiGuid {
        data1: 0x8868e871,
        data2: 0xe4f1,
        data3: 0x11d3,
        data4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
    };

    fn has_correct_signature(&self) -> bool {
        self.signature == u64::from_le_bytes(*b"RSD PTR ")
    }

    unsafe fn xsdt(&self) -> &AcpiExtendedSystemDescriptionTable {
        &*(self.xsdt_address as usize as *const AcpiExtendedSystemDescriptionTable)
    }
}

#[repr(C)]
#[derive(Debug)]
struct AcpiRootSystemDescriptionTable {
    signature: u32,
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id_lo: u32,
    oem_table_id_hi: u32,
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
    entry: [u32; 0],
}
impl AcpiRootSystemDescriptionTable {
    fn has_correct_signature(&self) -> bool {
        self.signature == u32::from_le_bytes(*b"RSDT")
    }

    fn entries(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.entry.as_ptr(), (self.length as usize - 36) / 4) }
    }
}

#[repr(C)]
#[derive(Debug)]
struct AcpiExtendedSystemDescriptionTable {
    signature: u32,
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: u64,
    oem_revision_id: u32,
    creator_id: u32,
    creator_revision: u32,
    // Note: offset=36に配置する必要があるのでアラインメントを4にしないといけない（u64だと8でずれる）
    entry: [[u32; 2]; 0],
}
impl AcpiExtendedSystemDescriptionTable {
    fn has_correct_signature(&self) -> bool {
        self.signature == u32::from_le_bytes(*b"XSDT")
    }

    fn entries(&self) -> &[u64] {
        unsafe {
            core::slice::from_raw_parts(self.entry.as_ptr() as _, (self.length as usize - 36) / 8)
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct AcpiSystemDescriptionTableHeader {
    signature: u32,
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    // Note: for 4-byte alignment
    oem_table_id: [u32; 2],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}
impl AcpiSystemDescriptionTableHeader {
    fn signature_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::mem::transmute::<_, &[u8; 4]>(&self.signature))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct AcpiFixedDescriptionTable {
    header: AcpiSystemDescriptionTableHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    _reserved: u8,
    preferred_pm_profile: u8,
    sci_int: u16,
    smi_cmd: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_cnt: u8,
    pm1a_evt_blk: u32,
    pm1b_evt_blk: u32,
    pm1a_cnt_blk: u32,
    pm1b_cnt_blk: u32,
    pm2_cnt_blk: u32,
    pm_tmr_blk: u32,
    gpe0_blk: u32,
    gpe1_blk: u32,
    pm1_evt_len: u8,
    pm1_cnt_len: u8,
    pm2_cnt_len: u8,
    pm_tmr_len: u8,
    gpe0_blk_len: u8,
    gpe1_blk_len: u8,
    gpe1_base: u8,
    cst_cnt: u8,
    p_lvl2_lat: u16,
    p_lvl3_lat: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alrm: u8,
    mon_alrm: u8,
    century: u8,
    // Note: for 1-byte alignment
    iapc_boot_arch: [u8; 2],
    _resv2: u8,
    flags: u32,
    reset_reg: [u8; 12],
    reset_value: u8,
    // Note: for 1-byte alignment
    arm_boot_arch: [u8; 2],
    fadt_minor_version: u8,
    // Note: for 4-byte alignment
    x_firmware_ctrl: [u32; 2],
    // Note: for 4-byte alignment
    x_dsdt: [u32; 2],
    x_pm1a_evt_blk: [u8; 12],
    x_pm1b_evt_blk: [u8; 12],
    x_pm1a_cnt_blk: [u8; 12],
    x_pm1b_cnt_blk: [u8; 12],
    x_pm2_cnt_blk: [u8; 12],
    x_pm_tmr_blk: [u8; 12],
    x_gpe0_blk: [u8; 12],
    x_gpe1_blk: [u8; 12],
    sleep_control_reg: [u8; 12],
    sleep_status_reg: [u8; 12],
    // Note: for 4-byte alignment
    hypervisor_vendor_identity: [u32; 2],
}
impl AcpiFixedDescriptionTable {
    const SIGNATURE: u32 = u32::from_ne_bytes(*b"FACP");
}

#[repr(C)]
struct AcpiMultipleAPICDescriptionTable {
    header: AcpiSystemDescriptionTableHeader,
    local_interrupt_controller_address: u32,
    flags: MultipleAPICDescriptionTableFlags,
    interrupt_controller_structure: [u8; 0],
}
impl AcpiMultipleAPICDescriptionTable {
    const SIGNATURE: u32 = u32::from_ne_bytes(*b"APIC");

    fn interrupt_controller_structure_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.interrupt_controller_structure.as_ptr(),
                self.header.length as usize - 44,
            )
        }
    }
}

#[repr(transparent)]
struct MultipleAPICDescriptionTableFlags(u32);
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
struct ProcessorLocalAPICStructure {
    r#type: u8,
    length: u8,
    acpi_processor_uid: u8,
    apic_id: u8,
    flags: u32,
}
impl ProcessorLocalAPICStructure {
    const TYPE: u8 = 0x00;
}

#[repr(C)]
struct IOAPICStructure {
    r#type: u8,
    length: u8,
    io_apic_id: u8,
    _resv: u8,
    io_apic_address: u32,
    global_system_interrupt_base: u32,
}
impl IOAPICStructure {
    const TYPE: u8 = 0x01;
}

#[repr(C)]
struct InterruptSourceOverrideStructure {
    r#type: u8,
    length: u8,
    bus: u8,
    source: u8,
    global_system_interrupt: u32,
    flags: InterruptSourceOverrideFlags,
}
impl InterruptSourceOverrideStructure {
    const TYPE: u8 = 0x02;
}

#[repr(transparent)]
struct InterruptSourceOverrideFlags(u16);
impl core::fmt::Debug for InterruptSourceOverrideFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let polarity = match self.0 & 0x03 {
            0 => "Conform",
            1 => "Active High",
            2 => "<Reserved>",
            3 => "Active Low",
            _ => unreachable!(),
        };
        let trigger_mode = match (self.0 >> 2) & 0x03 {
            0 => "Conform",
            1 => "Edge",
            2 => "<Reserved>",
            3 => "Level",
            _ => unreachable!(),
        };
        let rest = self.0 >> 4;

        write!(f, "Polarity({polarity}), TriggerMode({trigger_mode})")?;
        if rest != 0 {
            write!(f, ", Rest({rest})")?;
        }

        Ok(())
    }
}

#[repr(C)]
struct LocalAPICNMIStructure {
    r#type: u8,
    length: u8,
    acpi_processor_uid: u8,
    flags: [u8; 2],
    local_apic_lint_number: u8,
}
impl LocalAPICNMIStructure {
    const TYPE: u8 = 0x04;
}

struct ConsoleWriter {
    protocol: *mut EfiSimpleTextOutputProtocol,
}
impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut text_u16 = [0u16; 128];
        let mut write_ptr = 0;

        fn push_char<const L: usize>(
            this: &mut ConsoleWriter,
            sink: &mut [u16; L],
            write_ptr: &mut usize,
            c: char,
        ) {
            if *write_ptr + c.len_utf16() >= L {
                // flushing
                unsafe {
                    ((*this.protocol).output_string)(this.protocol, sink.as_mut_ptr());
                }
                sink.fill(0);
                *write_ptr = 0;
            }

            *write_ptr += c.encode_utf16(&mut sink[*write_ptr..]).len();
        }

        for c in s.chars() {
            if c == '\n' {
                push_char(self, &mut text_u16, &mut write_ptr, '\r');
            }
            push_char(self, &mut text_u16, &mut write_ptr, c);
        }

        if write_ptr > 0 {
            unsafe {
                ((*self.protocol).output_string)(self.protocol, text_u16.as_mut_ptr());
            }
        }

        Ok(())
    }
}

const ARROW_BITMAP: &'static [[u8; 16]; 16] = &[
    *b"@               ",
    *b"@@              ",
    *b"@.@             ",
    *b"@..@            ",
    *b"@...@           ",
    *b"@....@          ",
    *b"@.....@         ",
    *b"@......@        ",
    *b"@.......@       ",
    *b"@........@      ",
    *b"@@@..@@@@@@     ",
    *b"  @..@          ",
    *b"  @..@          ",
    *b"   @..@         ",
    *b"   @..@         ",
    *b"    @@          ",
];

#[no_mangle]
fn efi_main(_efi_handle: *mut core::ffi::c_void, system_table: *mut EfiSystemTable) {
    unsafe {
        SYSTEM_TABLE = system_table;
    }

    let mut con_out = ConsoleWriter {
        protocol: unsafe { (*system_table).con_out },
    };

    writeln!(
        &mut con_out,
        "Hello world!\r\nconfiguration tables: {}",
        unsafe { (*system_table).number_of_table_entries }
    )
    .unwrap();

    let configuration_tables = unsafe {
        core::slice::from_raw_parts(
            (*system_table).configuration_table,
            (*system_table).number_of_table_entries,
        )
    };
    for cfg in configuration_tables {
        writeln!(
            &mut con_out,
            "* {} = {:?}",
            cfg.vendor_guid, cfg.vendor_table
        )
        .unwrap();

        // if cfg.vendor_guid == AcpiRootSystemDescriptionPointer::GUID2 {
        //     let s = unsafe { &*(cfg.vendor_table as *mut AcpiRootSystemDescriptionPointer) };
        //     if !s.has_correct_signature() {
        //         panic!("invalid rsdt signature?");
        //     }
        //     writeln!(&mut con_out, "ACPI RSDP Structure: {s:?}").unwrap();

        //     if s.revision >= 2 {
        //         // acpi 2.0
        //         let table = unsafe { s.xsdt() };
        //         if !table.has_correct_signature() {
        //             panic!("invalid xsdt table signature?");
        //         }
        //         writeln!(&mut con_out, "XSDT: {table:?}").unwrap();
        //         writeln!(&mut con_out, "- oem_id: {}", unsafe {
        //             core::str::from_utf8_unchecked(&table.oem_id)
        //         })
        //         .unwrap();
        //         for e in table.entries() {
        //             writeln!(&mut con_out, "- entry: {e:016x}").unwrap();
        //             let child_table =
        //                 unsafe { &*(*e as usize as *const AcpiSystemDescriptionTableHeader) };
        //             writeln!(&mut con_out, "  - sig: {}", child_table.signature_str()).unwrap();

        //             if child_table.signature == AcpiFixedDescriptionTable::SIGNATURE {
        //                 let fixed_dt = unsafe {
        //                     &*(child_table as *const _ as *const AcpiFixedDescriptionTable)
        //                 };
        //                 writeln!(&mut con_out, "  - fixed table: {fixed_dt:?}").unwrap();
        //             }

        //             if child_table.signature == AcpiMultipleAPICDescriptionTable::SIGNATURE {
        //                 let t = unsafe {
        //                     &*(child_table as *const _ as *const AcpiMultipleAPICDescriptionTable)
        //                 };
        //                 writeln!(
        //                     &mut con_out,
        //                     "  - local interrupt controller address: 0x{:08x}",
        //                     t.local_interrupt_controller_address
        //                 )
        //                 .unwrap();
        //                 writeln!(&mut con_out, "  - flags: {:?}", t.flags).unwrap();

        //                 let ic = t.interrupt_controller_structure_bytes();
        //                 let mut ic_ptr = 0;
        //                 while ic_ptr < ic.len() {
        //                     let head = ic_ptr;
        //                     let type_byte = ic[ic_ptr];
        //                     ic_ptr += 1;
        //                     let length = ic[ic_ptr];
        //                     ic_ptr += 1;
        //                     writeln!(
        //                         &mut con_out,
        //                         "  - interrupt controller: 0x{type_byte:02x} len={length}"
        //                     )
        //                     .unwrap();
        //                     match type_byte {
        //                         ProcessorLocalAPICStructure::TYPE => {
        //                             let s = unsafe {
        //                                 &*(ic.as_ptr().add(head)
        //                                     as *const ProcessorLocalAPICStructure)
        //                             };
        //                             writeln!(
        //                                 &mut con_out,
        //                                 "    - processor_uid={},id={},flags={:x}",
        //                                 s.acpi_processor_uid, s.apic_id, s.flags
        //                             )
        //                             .unwrap();
        //                         }
        //                         IOAPICStructure::TYPE => {
        //                             let s = unsafe {
        //                                 &*(ic.as_ptr().add(head) as *const IOAPICStructure)
        //                             };
        //                             writeln!(
        //                                 &mut con_out,
        //                                 "    - id={},addr=0x{:08x},gsi_base={}",
        //                                 s.io_apic_id,
        //                                 s.io_apic_address,
        //                                 s.global_system_interrupt_base
        //                             )
        //                             .unwrap();
        //                         }
        //                         InterruptSourceOverrideStructure::TYPE => {
        //                             let s = unsafe {
        //                                 &*(ic.as_ptr().add(head)
        //                                     as *const InterruptSourceOverrideStructure)
        //                             };
        //                             writeln!(
        //                                 &mut con_out,
        //                                 "    - bus={},source={},gsi={},flags={:?}",
        //                                 s.bus, s.source, s.global_system_interrupt, s.flags
        //                             )
        //                             .unwrap();
        //                         }
        //                         LocalAPICNMIStructure::TYPE => {
        //                             let s = unsafe {
        //                                 &*(ic.as_ptr().add(head) as *const LocalAPICNMIStructure)
        //                             };
        //                             writeln!(
        //                                 &mut con_out,
        //                                 "    - processor_uid={},flags={:?},lint={}",
        //                                 s.acpi_processor_uid, s.flags, s.local_apic_lint_number
        //                             )
        //                             .unwrap();
        //                         }
        //                         _ => (),
        //                     }
        //                     ic_ptr += length as usize - 2;
        //                 }
        //             }
        //         }
        //     } else {
        //         let table =
        //             unsafe { &*(s.rsdt_address as usize as *mut AcpiRootSystemDescriptionTable) };
        //         if !table.has_correct_signature() {
        //             panic!("invalid rsdt table signature?");
        //         }

        //         writeln!(&mut con_out, "RSDT: {table:?}").unwrap();
        //         writeln!(&mut con_out, "- oem_id: {}", unsafe {
        //             core::str::from_utf8_unchecked(&table.oem_id)
        //         })
        //         .unwrap();
        //         for e in table.entries() {
        //             writeln!(&mut con_out, "- entry: {e:08x}").unwrap();
        //         }
        //     }
        // }

        if cfg.vendor_guid == EfiMemoryAttributeTable::GUID {
            let table = cfg.vendor_table as *mut EfiMemoryAttributeTable;
            writeln!(
                &mut con_out,
                "Memory Attribute: count={} ds={} flags={:02x}",
                unsafe { (*table).number_of_entries },
                unsafe { (*table).descriptor_size },
                unsafe { (*table).flags }
            )
            .unwrap();
            // let descriptors = unsafe {
            //     core::slice::from_raw_parts(
            //         (*table).entries.as_ptr(),
            //         (*table).number_of_entries as _,
            //     )
            // };
            // for d in descriptors {
            // なんかサイズちがうっぽい？のかずれて表示されていそう
            //     writeln!(
            //         &mut con_out,
            //         "-- {:016x}({:016x}): type={:08x} pc={} attr={:x}",
            //         d.physical_start, d.virtual_start, d.r#type, d.number_of_pages, d.attribute
            //     )
            //     .unwrap();
            // }
        }
    }

    let mut gop = core::ptr::null_mut::<EfiGraphicsOutputProtocol>();
    let r = unsafe {
        ((&*(&*system_table).boot_services).locate_protocol)(
            &EfiGraphicsOutputProtocol::GUID,
            core::ptr::null(),
            &mut gop as *mut _ as _,
        )
    };
    if r != 0 {
        panic!("Failed to locate gop: {r}");
    }

    let current_info = unsafe { &*(&*((&*gop).mode)).info };
    writeln!(&mut con_out, "current graphics mode:").unwrap();
    writeln!(
        &mut con_out,
        "- res: {}x{}",
        current_info.horizontal_resolution, current_info.vertical_resolution
    )
    .unwrap();
    writeln!(
        &mut con_out,
        "- pixel format: {:?} bitmask={:?}",
        current_info.pixel_format, current_info.pixel_information
    )
    .unwrap();
    writeln!(
        &mut con_out,
        "- pixels per scan-line: {}",
        current_info.pixels_per_scan_line
    )
    .unwrap();

    writeln!(&mut con_out, "Enumerating graphics mode:").unwrap();
    for n in 0..unsafe { (&*(&*gop).mode).max_mode } {
        let mut info = core::ptr::null_mut::<EfiGraphicsOutputModeInformation>();
        let mut info_size = core::mem::size_of::<EfiGraphicsOutputModeInformation>();
        let r = unsafe { ((&*gop).query_mode)(gop, n, &mut info_size, &mut info) };
        if r != 0 {
            panic!("Failed to query mode: {r}");
        }

        let info = unsafe { &*info };
        writeln!(
            &mut con_out,
            "- #{n}: {}x{} {:?} {:?} {}",
            info.horizontal_resolution,
            info.vertical_resolution,
            info.pixel_format,
            info.pixel_information,
            info.pixels_per_scan_line
        )
        .unwrap();
    }

    // let r = unsafe { ((*gop).set_mode)(gop, 0) };
    // if r != 0 {
    //     panic!("Unable to set graphics mode: {r}");
    // }

    let mut arrow_blt_buffer = [EfiGraphicsOutputBltPixel {
        red: 0,
        green: 0,
        blue: 0,
        reserved: 0,
    }; 16 * 16];
    for (y, r) in ARROW_BITMAP.into_iter().enumerate() {
        for (x, c) in r.into_iter().enumerate() {
            arrow_blt_buffer[x + y * 16] = match c {
                b'@' => EfiGraphicsOutputBltPixel {
                    red: 0,
                    green: 0,
                    blue: 0,
                    reserved: 255,
                },
                b'.' => EfiGraphicsOutputBltPixel {
                    red: 255,
                    green: 255,
                    blue: 255,
                    reserved: 255,
                },
                _ => EfiGraphicsOutputBltPixel {
                    red: 0,
                    green: 128,
                    blue: 128,
                    reserved: 255,
                },
            };
        }
    }
    let r = unsafe {
        ((*gop).blt)(
            gop,
            arrow_blt_buffer.as_mut_ptr(),
            EfiGraphicsOutputBltOperation::BltBufferToVideo,
            0,
            0,
            0,
            0,
            16,
            16,
            0,
        )
    };
    if r != 0 {
        panic!("Failed to blt arrow: {r}");
    }

    loop {}
}
