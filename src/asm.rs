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
