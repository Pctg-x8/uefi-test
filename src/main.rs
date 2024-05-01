#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

mod acpi;
mod asm;
mod hires_console;
mod pci;
mod uefi;
mod virtio;
use hires_console::HiResConsole;

static mut SYSTEM_TABLE: *mut uefi::EfiSystemTable = core::ptr::null_mut();
static mut HIRES_CONSOLE: *mut HiResConsole = core::ptr::null_mut();

#[panic_handler]
fn panic_handler<'a, 'b>(info: &'a PanicInfo<'b>) -> ! {
    let hires_console = unsafe { HIRES_CONSOLE };
    if !hires_console.is_null() {
        // use hires console as output
        writeln!(unsafe { &mut *hires_console }, "[PANIC OCCURRED] {info}").unwrap();

        loop {}
    }

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

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SegmentDescriptor(pub u64);
impl SegmentDescriptor {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn new_64bit_hi_base(base_hi32: u32) -> Self {
        Self(base_hi32 as _)
    }

    pub const fn base_address(self, a: u32) -> Self {
        let base_addr_lo16 = (a & 0xffff) as u64;
        let base_addr_md8 = ((a >> 16) & 0xff) as u64;
        let base_addr_hi8 = ((a >> 24) & 0xff) as u64;
        const CLEAR_MASK: u64 = !0xff00_00ff_ffff_0000;

        Self(
            (self.0 & CLEAR_MASK)
                | (base_addr_hi8 << 56)
                | (base_addr_md8 << 32)
                | (base_addr_lo16 << 16),
        )
    }

    pub const fn limit(self, lim: u32, large: bool) -> Self {
        let lim_lo16 = (lim & 0xffff) as u64;
        let lim_hi4 = ((lim >> 16) & 0x0f) as u64;
        let large_bit = if large { 1u64 } else { 0u64 };
        const CLEAR_MASK: u64 = !0x008f_0000_0000_ffff;

        Self((self.0 & CLEAR_MASK) | (lim_lo16) | (lim_hi4 << 48) | (large_bit << 55))
    }

    pub const fn present(self) -> Self {
        Self(self.0 | (1u64 << 47))
    }

    pub const fn privilege_level(self, level: u8) -> Self {
        let level = (level & 0x03) as u64;
        const CLEAR_MASK: u64 = !0x0000_6000_0000_0000;

        Self((self.0 & CLEAR_MASK) | (level << 45))
    }

    pub const fn r#type(self, r#type: u8) -> Self {
        let r#type = (r#type & 0x0f) as u64;
        const CLEAR_MASK: u64 = !0x0000_0f00_0000_0000;

        Self((self.0 & CLEAR_MASK) | (r#type << 40))
    }

    pub const fn code_64bit(self) -> Self {
        Self(self.0 | (1 << 53))
    }

    pub const fn default_operation_32bit(self) -> Self {
        Self(self.0 | (1 << 54))
    }

    pub const fn for_normal_code_data_segment(self) -> Self {
        Self(self.0 | (1 << 44))
    }
}

const GDT_PLACEMENT: *mut SegmentDescriptor = 0x0010_0000 as usize as _;
const GDT_ENTRY_COUNT: u16 = 8192;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SegmentSelector(pub u16);
impl SegmentSelector {
    pub const fn global(index: u16) -> Self {
        Self(index << 3)
    }

    pub const fn local(index: u16) -> Self {
        Self((index << 3) | (1 << 2))
    }

    pub const fn requested_privilege_level(self, level: u8) -> Self {
        Self((self.0 & !0x03) | (level as u16 & 0x03))
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct InterruptGateDescriptor(pub [u64; 2]);
impl InterruptGateDescriptor {
    pub const EMPTY: Self = Self([0; 2]);

    pub const fn new_interrupt(segment: SegmentSelector, addr: u64) -> Self {
        let (addr_hi16, addr_lo16) = ((addr >> 16) & 0xffff, addr & 0xffff);

        Self([
            addr >> 32,
            (addr_hi16 << 48) | (0b00110000 << 37) | ((segment.0 as u64) << 16) | (addr_lo16),
        ])
        .present()
    }

    pub const fn privilege_level(self, level: u8) -> Self {
        let level = (level & 0x03) as u64;
        const CLEAR_MASK: u64 = !0x0000_6000_0000_0000;

        Self([self.0[0], (self.0[1] & CLEAR_MASK) | (level << 45)])
    }

    pub const fn present(self) -> Self {
        Self([self.0[0], self.0[1] | (1 << 47)])
    }

    pub const fn size_32bit(self) -> Self {
        Self([self.0[0], self.0[1] | (1 << 43)])
    }
}

// Note: {pointer}::addは使えなかった（コンパイルエラーになる）
const IDT_PLACEMENT: *mut InterruptGateDescriptor = unsafe {
    (core::mem::transmute::<_, usize>(GDT_PLACEMENT)
        + core::mem::size_of::<SegmentDescriptor>() * GDT_ENTRY_COUNT as usize) as _
};
const IDT_ENTRY_COUNT: u16 = 256;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ControlRegister3(pub u64);
impl ControlRegister3 {
    pub const fn new(paging_root_table_phys_address: u64) -> Self {
        assert!(
            paging_root_table_phys_address & 0xfff == 0,
            "paging root table is not aligned by 4k"
        );

        Self(paging_root_table_phys_address & !0xfff)
    }

    pub const fn write_through(self) -> Self {
        Self(self.0 | (1 << 3))
    }

    pub const fn cache_disable(self) -> Self {
        Self(self.0 | (1 << 4))
    }

    pub fn store(self) {
        unsafe { store_cr!(3, self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PML4Entry(pub u64);
impl PML4Entry {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn new(page_directory_pointer_table_phys_address: u64) -> Self {
        assert!(
            page_directory_pointer_table_phys_address & 0xfff == 0,
            "page directory pointer table is not aligned by 4k"
        );

        // set with present flag
        Self((page_directory_pointer_table_phys_address & !0xfff) | 0x01)
    }

    #[inline]
    pub const fn writable(self) -> Self {
        Self(self.0 | 0x02)
    }

    #[inline]
    pub const fn allow_user(self) -> Self {
        Self(self.0 | 0x04)
    }

    #[inline]
    pub const fn write_through(self) -> Self {
        Self(self.0 | 0x08)
    }

    #[inline]
    pub const fn cache_disable(self) -> Self {
        Self(self.0 | 0x10)
    }

    #[inline]
    pub const fn execute_disable(self) -> Self {
        Self(self.0 | 0x8000_0000_0000_0000)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageDirectoryPointerTableEntry(pub u64);
impl PageDirectoryPointerTableEntry {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn new(page_directory_phys_address: u64) -> Self {
        assert!(
            page_directory_phys_address & 0xfff == 0,
            "Page Directory is not aligned by 4k"
        );

        // set with present flag
        Self((page_directory_phys_address & !0xfff) & 0x01)
    }

    #[inline]
    pub const fn writable(self) -> Self {
        Self(self.0 | 0x02)
    }

    #[inline]
    pub const fn allow_user(self) -> Self {
        Self(self.0 | 0x04)
    }

    #[inline]
    pub const fn write_through(self) -> Self {
        Self(self.0 | 0x08)
    }

    #[inline]
    pub const fn cache_disable(self) -> Self {
        Self(self.0 | 0x10)
    }

    #[inline]
    pub const fn execute_disable(self) -> Self {
        Self(self.0 | 0x8000_0000_0000_0000)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageDirectoryEntry(pub u64);
impl PageDirectoryEntry {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn new(page_table_phys_address: u64) -> Self {
        assert!(
            page_table_phys_address & 0xfff == 0,
            "Page Table is not aligned by 4k"
        );

        // set with present flag
        Self((page_table_phys_address & !0xfff) | 0x01)
    }

    #[inline]
    pub const fn writable(self) -> Self {
        Self(self.0 | 0x02)
    }

    #[inline]
    pub const fn allow_user(self) -> Self {
        Self(self.0 | 0x04)
    }

    #[inline]
    pub const fn write_through(self) -> Self {
        Self(self.0 | 0x08)
    }

    #[inline]
    pub const fn cache_disable(self) -> Self {
        Self(self.0 | 0x10)
    }

    #[inline]
    pub const fn execute_disable(self) -> Self {
        Self(self.0 | 0x8000_0000_0000_0000)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(pub u64);
impl PageTableEntry {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn new(page_phys_address: u64) -> Self {
        assert!(page_phys_address & 0xfff == 0, "Page is not aligned by 4k");

        // set with present flag
        Self((page_phys_address & !0xfff) | 0x01)
    }

    #[inline]
    pub const fn writable(self) -> Self {
        Self(self.0 | 0x02)
    }

    #[inline]
    pub const fn allow_user(self) -> Self {
        Self(self.0 | 0x04)
    }

    #[inline]
    pub const fn write_through(self) -> Self {
        Self(self.0 | 0x08)
    }

    #[inline]
    pub const fn cache_disable(self) -> Self {
        Self(self.0 | 0x10)
    }

    #[inline]
    pub const fn pat(self) -> Self {
        Self(self.0 | 0x80)
    }

    #[inline]
    pub const fn global(self) -> Self {
        Self(self.0 | 0x100)
    }

    #[inline]
    pub const fn protection_key(self, key: u8) -> Self {
        Self((self.0 & !(0x0f << 59)) | ((key as u64 & 0x0f) << 59))
    }

    #[inline]
    pub const fn execute_disable(self) -> Self {
        Self(self.0 | 0x8000_0000_0000_0000)
    }
}

#[no_mangle]
fn efi_main(efi_handle: uefi::EfiHandle, system_table: *mut uefi::EfiSystemTable) {
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

        if cfg.vendor_guid == acpi::RootSystemDescriptionPointer::GUID_V2 {
            let s = unsafe { &*(cfg.vendor_table as *mut acpi::RootSystemDescriptionPointer) };
            if !s.has_correct_signature() {
                panic!("invalid rsdt signature?");
            }
            writeln!(&mut con_out, "ACPI RSDP Structure: {s:?}").unwrap();

            if s.revision >= 2 {
                // acpi 2.0
                let table = unsafe { s.xsdt() };
                if !table.has_correct_signature() {
                    panic!("invalid xsdt table signature?");
                }
                writeln!(&mut con_out, "XSDT: {table:?}").unwrap();
                writeln!(&mut con_out, "- oem_id: {}", unsafe {
                    core::str::from_utf8_unchecked(&table.oem_id)
                })
                .unwrap();
                for e in table.entries() {
                    writeln!(&mut con_out, "- entry: {e:016x}").unwrap();
                    let child_table =
                        unsafe { &*(*e as usize as *const acpi::SystemDescriptionTableHeader) };
                    writeln!(&mut con_out, "  - sig: {}", child_table.signature_str()).unwrap();

                    if child_table.signature == acpi::FixedDescriptionTable::SIGNATURE {
                        let fixed_dt = unsafe {
                            &*(child_table as *const _ as *const acpi::FixedDescriptionTable)
                        };
                        writeln!(&mut con_out, "  - fixed table: {fixed_dt:?}").unwrap();
                    }

                    if child_table.signature == acpi::MultipleAPICDescriptionTable::SIGNATURE {
                        let t = unsafe {
                            &*(child_table as *const _ as *const acpi::MultipleAPICDescriptionTable)
                        };
                        writeln!(
                            &mut con_out,
                            "  - local interrupt controller address: 0x{:08x}",
                            t.local_interrupt_controller_address
                        )
                        .unwrap();
                        writeln!(&mut con_out, "  - flags: {:?}", t.flags).unwrap();

                        let ic = t.interrupt_controller_structure_bytes();
                        let mut ic_ptr = 0;
                        while ic_ptr < ic.len() {
                            let head = ic_ptr;
                            let type_byte = ic[ic_ptr];
                            ic_ptr += 1;
                            let length = ic[ic_ptr];
                            ic_ptr += 1;
                            writeln!(
                                &mut con_out,
                                "  - interrupt controller: 0x{type_byte:02x} len={length}"
                            )
                            .unwrap();
                            match type_byte {
                                acpi::ProcessorLocalAPICStructure::TYPE => {
                                    let s = unsafe {
                                        &*(ic.as_ptr().add(head)
                                            as *const acpi::ProcessorLocalAPICStructure)
                                    };
                                    writeln!(
                                        &mut con_out,
                                        "    - local apic: processor_uid={},id={},flags={:x}",
                                        s.acpi_processor_uid, s.apic_id, s.flags
                                    )
                                    .unwrap();
                                }
                                acpi::IOAPICStructure::TYPE => {
                                    let s = unsafe {
                                        &*(ic.as_ptr().add(head) as *const acpi::IOAPICStructure)
                                    };
                                    writeln!(
                                        &mut con_out,
                                        "    - io apic: id={},addr=0x{:08x},gsi_base={}",
                                        s.io_apic_id,
                                        s.io_apic_address,
                                        s.global_system_interrupt_base
                                    )
                                    .unwrap();
                                }
                                acpi::InterruptSourceOverrideStructure::TYPE => {
                                    let s = unsafe {
                                        &*(ic.as_ptr().add(head)
                                            as *const acpi::InterruptSourceOverrideStructure)
                                    };
                                    writeln!(
                                        &mut con_out,
                                        "    - iso: bus={},source={},gsi={},flags={:?}",
                                        s.bus, s.source, s.global_system_interrupt, s.flags
                                    )
                                    .unwrap();
                                }
                                acpi::LocalAPICNMIStructure::TYPE => {
                                    let s = unsafe {
                                        &*(ic.as_ptr().add(head)
                                            as *const acpi::LocalAPICNMIStructure)
                                    };
                                    writeln!(
                                        &mut con_out,
                                        "    - local apic nmi: processor_uid={},flags={:?},lint={}",
                                        s.acpi_processor_uid, s.flags, s.local_apic_lint_number
                                    )
                                    .unwrap();
                                }
                                _ => (),
                            }
                            ic_ptr += length as usize - 2;
                        }
                    }
                }
            } else {
                let table =
                    unsafe { &*(s.rsdt_address as usize as *mut acpi::RootSystemDescriptionTable) };
                if !table.has_correct_signature() {
                    panic!("invalid rsdt table signature?");
                }

                writeln!(&mut con_out, "RSDT: {table:?}").unwrap();
                writeln!(&mut con_out, "- oem_id: {}", unsafe {
                    core::str::from_utf8_unchecked(&table.oem_id)
                })
                .unwrap();
                for e in table.entries() {
                    writeln!(&mut con_out, "- entry: {e:08x}").unwrap();
                }
            }
        }

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

    // setup hires console
    let mut gop = core::ptr::null_mut::<uefi::EfiGraphicsOutputProtocol>();
    let r = unsafe {
        ((&*system_table.boot_services).locate_protocol)(
            &uefi::EfiGraphicsOutputProtocol::GUID,
            core::ptr::null(),
            &mut gop as *mut _ as _,
        )
    };
    if r != 0 {
        panic!("Failed to locate gop: 0x{r:016x}");
    }
    let gop = unsafe { &mut *gop };

    let Some((preferred_mode, mode_info)) = (0..gop.mode().max_mode)
        .map(|n| {
            let mut info = core::ptr::null_mut::<uefi::EfiGraphicsOutputModeInformation>();
            let mut info_size = core::mem::size_of::<uefi::EfiGraphicsOutputModeInformation>();
            let r = gop.query_mode(n, &mut info_size, &mut info);
            if r != 0 {
                panic!("Failed to query mode: {r}");
            }

            (n, unsafe { &*info })
        })
        .filter(|(_, x)| {
            (640..=1280).contains(&x.horizontal_resolution)
                && (x.pixel_format
                    == uefi::EfiGraphicsPixelFormat::BlueGreenRedReserved8BitPerColor
                    || x.pixel_format
                        == uefi::EfiGraphicsPixelFormat::RedGreenBlueReserved8BitPerColor)
        })
        .max_by_key(|(_, x)| (x.horizontal_resolution, x.vertical_resolution))
    else {
        panic!("no preferred graphics mode found");
    };
    let r = gop.set_mode(preferred_mode);
    if r != 0 {
        panic!("Failed to set graphics mode: 0x{r:016x}");
    }

    let framebuffer_base = unsafe {
        core::slice::from_raw_parts_mut(
            gop.mode().frame_buffer_base as usize as *mut [u8; 4],
            gop.mode().frame_buffer_size / 4,
        )
    };
    let framebuffer_stride = mode_info.pixels_per_scan_line;
    let mut hrc = HiResConsole::new(
        framebuffer_base,
        framebuffer_stride,
        mode_info.vertical_resolution,
    );
    unsafe {
        HIRES_CONSOLE = &mut hrc as _;
    }
    writeln!(
        &mut hrc,
        "HiResConsole Launched: ScreenRes={}x{}",
        mode_info.horizontal_resolution, mode_info.vertical_resolution
    )
    .unwrap();

    let mut memory_map_size = 0;
    let mut memory_map = [];
    let mut map_key = 0;
    let mut descriptor_size = 0;
    let mut descriptor_version = 0;
    let r = unsafe {
        ((&*system_table.boot_services).get_memory_map)(
            &mut memory_map_size,
            memory_map.as_mut_ptr(),
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        )
    };
    if r != 0 && r != 0x8000_0000_0000_0005 {
        panic!("GetMemoryMap failed: 0x{r:016x}");
    }

    let r = unsafe { ((&*system_table.boot_services).exit_boot_services)(efi_handle, map_key) };
    if r != 0 {
        panic!("ExitBootServices failed: 0x{r:016x}");
    }

    unsafe {
        cli!();
    }

    // paging state
    let cr0 = unsafe { load_cr!(0) };
    let cr4 = unsafe { load_cr!(4) };
    let efer = unsafe { rdmsr!(efer) };
    writeln!(
        &mut hrc,
        "paging state: EFER.LMA={lma}, EFER.LME={lme}, CR0.PG={pg}, CR4.PAE={pae}, CR4.LA57={la57}",
        lma = (efer & 0x400) != 0,
        lme = (efer & 0x100) != 0,
        pg = (cr0 & 0x8000_0000) != 0,
        pae = (cr4 & 0x20) != 0,
        la57 = (cr4 & 0x1000) != 0
    )
    .unwrap();
    if (cr4 & 0x1000) != 0 {
        unimplemented!("Level-5 Paging support");
    }

    if (cr4 & 0x20000) != 0 {
        unimplemented!("pcid support");
    }

    // TODO: ページング再設定するならちゃんとカーネル切り離してロードしたほうがいい（OS Loader自体がどこに入るのかがこっちからはわからないからコードページ設定できない）

    let global_descriptor_table =
        unsafe { core::slice::from_raw_parts_mut(GDT_PLACEMENT, GDT_ENTRY_COUNT as _) };
    global_descriptor_table.fill(SegmentDescriptor::new());
    global_descriptor_table[1] = SegmentDescriptor::new()
        .base_address(0)
        .limit(u32::MAX, true)
        .present()
        .privilege_level(0)
        .r#type(0b1000)
        .code_64bit()
        .default_operation_32bit()
        .for_normal_code_data_segment();
    global_descriptor_table[2] = SegmentDescriptor::new()
        .base_address(0)
        .limit(u32::MAX, true)
        .present()
        .privilege_level(0)
        .r#type(0b0010)
        .code_64bit()
        .default_operation_32bit()
        .for_normal_code_data_segment();
    unsafe {
        lgdt!(GDT_PLACEMENT, GDT_ENTRY_COUNT - 1);
    }

    let interrupt_descriptor_table =
        unsafe { core::slice::from_raw_parts_mut(IDT_PLACEMENT, IDT_ENTRY_COUNT as _) };
    interrupt_descriptor_table.fill(InterruptGateDescriptor::EMPTY);
    interrupt_descriptor_table[13] = InterruptGateDescriptor::new_interrupt(
        SegmentSelector::global(1).requested_privilege_level(0),
        general_protection_fault as *const extern "system" fn() as _,
    )
    .privilege_level(0)
    .size_32bit();
    unsafe {
        lidt!(IDT_PLACEMENT, IDT_ENTRY_COUNT - 1);
    }

    struct LocalAPIC {
        base_address: usize,
    }
    impl LocalAPIC {
        pub fn get(con: &mut impl Write) -> Self {
            let apic_base = unsafe { rdmsr!(0x1b) };
            writeln!(con, "apic_base register: 0x{apic_base:016x}").unwrap();

            Self {
                base_address: apic_base as usize & !0xfff,
            }
        }

        pub fn read_version_register(&self) -> u32 {
            unsafe { core::ptr::read_volatile((self.base_address + 0x30) as *const u32) }
        }
    }

    let local_apic = LocalAPIC::get(&mut hrc);
    let v = local_apic.read_version_register();
    writeln!(&mut hrc, "local apic version: 0x{v:08x}").unwrap();

    loop {}

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

    // for n in 0..32 {
    //     let root_device = pci::DeviceIdentifier {
    //         bus: 0,
    //         device: n,
    //         function: 0,
    //     };

    //     let [vendor_id, device_id] = root_device.read_device_vendor_ids();
    //     if vendor_id == 0xffff {
    //         continue;
    //     }
    //     let [x, pif, subclass, cls] = root_device.read_class_pif_revision_values();
    //     let [_, _, ht, _] = root_device.read_bist_ht_lt_cls_values();
    //     writeln!(
    //         &mut con_out,
    //         "root pci s#{n}.0: 0x{device_id:04x} 0x{vendor_id:04x} header_type=0x{ht:02x} cls={cls}:{subclass} pif={pif} x={x}"
    //     )
    //     .unwrap();

    //     if (ht & 0x80) != 0 {
    //         // multifunction
    //         for f in 1..8 {
    //             let root_device = pci::DeviceIdentifier {
    //                 bus: 0,
    //                 device: n,
    //                 function: f,
    //             };

    //             let [vendor_id, device_id] = root_device.read_device_vendor_ids();
    //             if vendor_id == 0xffff {
    //                 continue;
    //             }
    //             let [x, pif, subclass, cls] = root_device.read_class_pif_revision_values();
    //             let [_, _, ht, _] = root_device.read_bist_ht_lt_cls_values();
    //             writeln!(
    //                 &mut con_out,
    //                 "root pci s#{n}.{f}: 0x{device_id:04x} 0x{vendor_id:04x} header_type=0x{ht:02x} cls={cls}:{subclass} pif={pif} x={x}"
    //             )
    //             .unwrap();
    //         }
    //     }

    //     if vendor_id == 0x1234 && device_id == 0x1111 {
    //         // QEMU/Bochs VGA Device
    //         let fb_address = root_device.read_base_address_register(0);
    //         let mmio_base = root_device.read_base_address_register(2);
    //         let endian = unsafe { *((mmio_base as usize + 0x0604) as *const u32) };
    //         writeln!(
    //             &mut con_out,
    //             "- VGA: fb=0x{fb_address:08x} mmio=0x{mmio_base:08x} end=0x{endian:08x}"
    //         )
    //         .unwrap();
    //     }

    //     if vendor_id == 0x1af4 && device_id == 0x1040 + 16 {
    //         // virtio-gpu
    //         let [_, st] = root_device.read_status_command_values();
    //         let cap = root_device.read_capabilities_pointer();
    //         writeln!(&mut con_out, "- Virtio GPU: st=0x{st:04x}, cap=0x{cap:02x}").unwrap();
    //         let mut cap_pointer = cap;
    //         while cap_pointer != 0 {
    //             let extra_config = root_device.read_config(cap_pointer >> 2);
    //             writeln!(&mut con_out, "- extra config: 0x{extra_config:08x}").unwrap();

    //             if extra_config & 0xff == 0x11 {
    //                 // msi-x caps
    //                 let message_control = (extra_config >> 16) as u16;
    //                 let (table_address, pending_bit_array_address) = (
    //                     root_device.read_config((cap_pointer >> 2) + 1),
    //                     root_device.read_config((cap_pointer >> 2) + 2),
    //                 );

    //                 writeln!(
    //                     &mut con_out,
    //                     "  - msi-x message control: 0x{message_control:04x}"
    //                 )
    //                 .unwrap();
    //                 writeln!(
    //                     &mut con_out,
    //                     "  - msi-x table address: 0x{table_address:08x}"
    //                 )
    //                 .unwrap();
    //                 writeln!(
    //                     &mut con_out,
    //                     "  - msi-x pending bit array address: 0x{pending_bit_array_address:08x}"
    //                 )
    //                 .unwrap();
    //             }
    //             if extra_config & 0xff == 0x09 {
    //                 // vendor specific: virtio caps
    //                 let ([_, _, _id, bar], offs, len) = (
    //                     root_device
    //                         .read_config((cap_pointer >> 2) + 1)
    //                         .to_le_bytes(),
    //                     root_device.read_config((cap_pointer >> 2) + 2),
    //                     root_device.read_config((cap_pointer >> 2) + 3),
    //                 );

    //                 let cap_type = (extra_config >> 24) as u8;
    //                 if cap_type == virtio::PCICapabilityType::CommonConfig as u8 {
    //                     writeln!(
    //                         &mut con_out,
    //                         "  - PCI Common Config at bar #{bar} +{offs} ~{len}"
    //                     )
    //                     .unwrap();

    //                     for n in 0..6 {
    //                         writeln!(
    //                             &mut con_out,
    //                             "  - bar #{n}: 0x{:08x}",
    //                             root_device.read_base_address_register(n as _)
    //                         )
    //                         .unwrap();
    //                     }

    //                     let common_config = unsafe {
    //                         &mut *((root_device.read_base_address_register(bar) + offs) as usize
    //                             as *mut virtio::CommonConfiguration)
    //                     };
    //                     writeln!(&mut con_out, "  - ptr: {common_config:p}").unwrap();
    //                     writeln!(&mut con_out, "  - cfg: {common_config:?}").unwrap();
    //                 }
    //             }

    //             cap_pointer = ((extra_config >> 8) & 0xfc) as u8;
    //         }
    //     }
    // }

    loop {}
}

extern "system" fn general_protection_fault() -> ! {
    writeln!(
        unsafe { &mut *HIRES_CONSOLE },
        "[ERR] General Protection Fault!"
    )
    .unwrap();
    loop {}
}
