#![no_std]
#![no_main]

use core::{ffi::c_void, fmt::Write, panic::PanicInfo};

#[panic_handler]
fn panic_handler<'a, 'b>(_info: &'a PanicInfo<'b>) -> ! {
    loop {}
}

type EfiHandle = *mut core::ffi::c_void;

#[repr(C)]
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
struct EfiBootServices {}

#[repr(C)]
struct EfiConfigurationTable {
    vendor_guid: EfiGuid,
    vendor_table: *mut core::ffi::c_void,
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

#[no_mangle]
fn efi_main(_efi_handle: *mut core::ffi::c_void, system_table: *mut EfiSystemTable) {
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
    }

    loop {}
}
