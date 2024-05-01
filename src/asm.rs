// Note: in/outはレジスタ指定が固定らしい
#[macro_export]
macro_rules! in32 {
    ($port: expr) => {{
        let res: u32;
        core::arch::asm!("in eax, dx", in("dx") $port, out("eax") res, options(nomem, nostack, preserves_flags));
        res
    }}
}
#[macro_export]
macro_rules! out32 {
    ($port: expr, $value: expr) => {
        core::arch::asm!("out dx, eax", in("dx") $port, in("eax") $value, options(nomem, nostack, preserves_flags));
    }
}

#[macro_export]
macro_rules! rdmsr {
    (efer) => {
        rdmsr!(0xc000_0080 as u32)
    };
    ($addr: expr) => {{
        let (hi, lo): (u32, u32);
        core::arch::asm!("rdmsr", in("ecx") $addr, out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
        (hi as u64) << 32 | lo as u64
    }};
}
#[macro_export]
macro_rules! wdmsr {
    ($addr: expr, $value: expr) => {{
        let v = $value;
        let (hi, lo) = ((v >> 32) as u32, v as u32);
        core::arch::asm!("wdmsr", in("ecx") $addr, in("edx") hi, in("eax") lo, options(nomem, nostack, preserves_flags));
    }}
}

#[macro_export]
macro_rules! lgdt {
    ($base_addr: expr, $limit: expr) => {{
        let mut content = [0u8; 10];
        content[2..].copy_from_slice(&u64::to_ne_bytes($base_addr as u64));
        content[..2].copy_from_slice(&u16::to_ne_bytes($limit));
        core::arch::asm!("lgdt [{content}]", content = in(reg) content.as_ptr(), options(nomem, nostack, preserves_flags));
    }};
}
#[macro_export]
macro_rules! lidt {
    ($base_addr: expr, $limit: expr) => {{
        let mut content = [0u8; 10];
        content[2..].copy_from_slice(&u64::to_ne_bytes($base_addr as _));
        content[..2].copy_from_slice(&u16::to_ne_bytes($limit as _));
        core::arch::asm!("lidt [{content}]", content = in(reg) content.as_ptr(), options(nomem, nostack, preserves_flags));
    }}
}

#[macro_export]
macro_rules! cli {
    () => {
        core::arch::asm!("cli");
    };
}

#[macro_export]
macro_rules! sti {
    () => {
        core::arch::asm!("sti");
    };
}

#[macro_export]
macro_rules! load_cr {
    (0) => {{
        let x: u64;
        core::arch::asm!("mov {dest:r}, cr0", dest = out(reg) x, options(nomem, nostack, preserves_flags));
        x
    }};
    (4) => {{
        let x: u64;
        core::arch::asm!("mov {dest:r}, cr4", dest = out(reg) x, options(nomem, nostack, preserves_flags));
        x
    }};
}

#[macro_export]
macro_rules! store_cr {
    (3, $value: expr) => {
        core::arch::asm!("mov cr3, {x}", x = in(reg) $value, options(nostack))
    }
}
