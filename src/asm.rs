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
    ($addr: expr) => {{
        let (hi, lo): (u32, u32);
        core::arch::asm!("rdmsr", in("ecx") $addr, out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
        (hi as u64) << 32 | lo as u64
    }}
}
#[macro_export]
macro_rules! wdmsr {
    ($addr: expr, $value: expr) => {{
        let v = $value;
        let (hi, lo) = ((v >> 32) as u32, v as u32);
        core::arch::asm!("wdmsr", in("ecx") $addr, in("edx") hi, in("eax") lo, options(nomem, nostack, preserves_flags));
    }}
}
