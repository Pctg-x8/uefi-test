#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

mod acpi;
mod asm;
mod pci;
mod uefi;
mod virtio;

static mut SYSTEM_TABLE: *mut uefi::EfiSystemTable = core::ptr::null_mut();

#[panic_handler]
fn panic_handler<'a, 'b>(info: &'a PanicInfo<'b>) -> ! {
    let mut con_out = ConsoleWriter {
        protocol: unsafe { (*SYSTEM_TABLE).con_out },
    };

    writeln!(&mut con_out, "[PANIC OCCURRED] {info}").unwrap();

    loop {}
}

struct ConsoleWriter {
    protocol: *mut uefi::EfiSimpleTextOutputProtocol,
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
fn efi_main(_efi_handle: *mut core::ffi::c_void, system_table: *mut uefi::EfiSystemTable) {
    unsafe {
        SYSTEM_TABLE = system_table;
    }

    let system_table = unsafe { &mut *system_table };
    let mut con_out = ConsoleWriter {
        protocol: system_table.con_out,
    };

    writeln!(
        &mut con_out,
        "Hello world!\r\nconfiguration tables: {}",
        system_table.number_of_table_entries
    )
    .unwrap();

    for cfg in system_table.configuration_table_entries() {
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

        if cfg.vendor_guid == uefi::EfiMemoryAttributeTable::GUID {
            let table = unsafe { &*(cfg.vendor_table as *mut uefi::EfiMemoryAttributeTable) };
            writeln!(
                &mut con_out,
                "Memory Attribute: count={} ds={} flags={:02x}",
                table.number_of_entries, table.descriptor_size, table.flags
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

    let mut gop = core::ptr::null_mut::<uefi::EfiGraphicsOutputProtocol>();
    let r = unsafe {
        ((&*system_table.boot_services).locate_protocol)(
            &uefi::EfiGraphicsOutputProtocol::GUID,
            core::ptr::null(),
            &mut gop as *mut _ as _,
        )
    };
    if r != 0 {
        panic!("Failed to locate gop: {r}");
    }
    let gop = unsafe { &mut *gop };

    let current_info = gop.mode().info();
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
    for n in 0..gop.mode().max_mode {
        let mut info = core::ptr::null_mut::<uefi::EfiGraphicsOutputModeInformation>();
        let mut info_size = core::mem::size_of::<uefi::EfiGraphicsOutputModeInformation>();
        let r = gop.query_mode(n, &mut info_size, &mut info);
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

    let mut arrow_blt_buffer = [uefi::EfiGraphicsOutputBltPixel {
        red: 0,
        green: 0,
        blue: 0,
        reserved: 0,
    }; 16 * 16];
    for (y, r) in ARROW_BITMAP.into_iter().enumerate() {
        for (x, c) in r.into_iter().enumerate() {
            arrow_blt_buffer[x + y * 16] = match c {
                b'@' => uefi::EfiGraphicsOutputBltPixel {
                    red: 0,
                    green: 0,
                    blue: 0,
                    reserved: 255,
                },
                b'.' => uefi::EfiGraphicsOutputBltPixel {
                    red: 255,
                    green: 255,
                    blue: 255,
                    reserved: 255,
                },
                _ => uefi::EfiGraphicsOutputBltPixel {
                    red: 0,
                    green: 128,
                    blue: 128,
                    reserved: 255,
                },
            };
        }
    }
    let r = gop.blt(
        &mut arrow_blt_buffer,
        uefi::EfiGraphicsOutputBltOperation::BltBufferToVideo,
        0,
        0,
        0,
        0,
        16,
        16,
        0,
    );
    if r != 0 {
        panic!("Failed to blt arrow: {r}");
    }

    for n in 0..32 {
        let root_device = pci::DeviceIdentifier {
            bus: 0,
            device: n,
            function: 0,
        };

        let [vendor_id, device_id] = root_device.read_device_vendor_ids();
        if vendor_id == 0xffff {
            continue;
        }
        let [x, pif, subclass, cls] = root_device.read_class_pif_revision_values();
        let [_, _, ht, _] = root_device.read_bist_ht_lt_cls_values();
        writeln!(
            &mut con_out,
            "root pci s#{n}.0: 0x{device_id:04x} 0x{vendor_id:04x} header_type=0x{ht:02x} cls={cls}:{subclass} pif={pif} x={x}"
        )
        .unwrap();

        if (ht & 0x80) != 0 {
            // multifunction
            for f in 1..8 {
                let root_device = pci::DeviceIdentifier {
                    bus: 0,
                    device: n,
                    function: f,
                };

                let [vendor_id, device_id] = root_device.read_device_vendor_ids();
                if vendor_id == 0xffff {
                    continue;
                }
                let [x, pif, subclass, cls] = root_device.read_class_pif_revision_values();
                let [_, _, ht, _] = root_device.read_bist_ht_lt_cls_values();
                writeln!(
                    &mut con_out,
                    "root pci s#{n}.{f}: 0x{device_id:04x} 0x{vendor_id:04x} header_type=0x{ht:02x} cls={cls}:{subclass} pif={pif} x={x}"
                )
                .unwrap();
            }
        }

        if vendor_id == 0x1234 && device_id == 0x1111 {
            // QEMU/Bochs VGA Device
            let fb_address = root_device.read_base_address_register(0);
            let mmio_base = root_device.read_base_address_register(2);
            let endian = unsafe { *((mmio_base as usize + 0x0604) as *const u32) };
            writeln!(
                &mut con_out,
                "- VGA: fb=0x{fb_address:08x} mmio=0x{mmio_base:08x} end=0x{endian:08x}"
            )
            .unwrap();
        }

        if vendor_id == 0x1af4 && device_id == 0x1040 + 16 {
            // virtio-gpu
            let [_, st] = root_device.read_status_command_values();
            let cap = root_device.read_capabilities_pointer();
            writeln!(&mut con_out, "- Virtio GPU: st=0x{st:04x}, cap=0x{cap:02x}").unwrap();
            let mut cap_pointer = cap;
            while cap_pointer != 0 {
                let extra_config = root_device.read_config(cap_pointer >> 2);
                writeln!(&mut con_out, "- extra config: 0x{extra_config:08x}").unwrap();

                if extra_config & 0xff == 0x11 {
                    // msi-x caps
                    let message_control = (extra_config >> 16) as u16;
                    let (table_address, pending_bit_array_address) = (
                        root_device.read_config((cap_pointer >> 2) + 1),
                        root_device.read_config((cap_pointer >> 2) + 2),
                    );

                    writeln!(
                        &mut con_out,
                        "  - msi-x message control: 0x{message_control:04x}"
                    )
                    .unwrap();
                    writeln!(
                        &mut con_out,
                        "  - msi-x table address: 0x{table_address:08x}"
                    )
                    .unwrap();
                    writeln!(
                        &mut con_out,
                        "  - msi-x pending bit array address: 0x{pending_bit_array_address:08x}"
                    )
                    .unwrap();
                }
                if extra_config & 0xff == 0x09 {
                    // vendor specific: virtio caps
                    let ([_, _, _id, bar], offs, len) = (
                        root_device
                            .read_config((cap_pointer >> 2) + 1)
                            .to_le_bytes(),
                        root_device.read_config((cap_pointer >> 2) + 2),
                        root_device.read_config((cap_pointer >> 2) + 3),
                    );

                    let cap_type = (extra_config >> 24) as u8;
                    if cap_type == virtio::PCICapabilityType::CommonConfig as u8 {
                        writeln!(
                            &mut con_out,
                            "  - PCI Common Config at bar #{bar} +{offs} ~{len}"
                        )
                        .unwrap();

                        for n in 0..6 {
                            writeln!(
                                &mut con_out,
                                "  - bar #{n}: 0x{:08x}",
                                root_device.read_base_address_register(n as _)
                            )
                            .unwrap();
                        }

                        let common_config = unsafe {
                            &mut *((root_device.read_base_address_register(bar) + offs) as usize
                                as *mut virtio::CommonConfiguration)
                        };
                        writeln!(&mut con_out, "  - ptr: {common_config:p}").unwrap();
                        writeln!(&mut con_out, "  - cfg: {common_config:?}").unwrap();
                    }
                }

                cap_pointer = ((extra_config >> 8) & 0xfc) as u8;
            }
        }
    }

    loop {}
}
