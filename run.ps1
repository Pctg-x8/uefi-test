$RustArtifactPath = "target/x86_64-unknown-uefi/debug/uefi-test.efi"
$BootloaderPath = "disk/EFI/BOOT/BOOTX64.efi"
$OvmfPath = "Z:\OVMF.fd"

cargo build --target x86_64-unknown-uefi
Copy-Item $RustArtifactPath $BootloaderPath
qemu-system-x86_64 -drive "if=pflash,format=raw,file=$OvmfPath" -drive "if=ide,index=0,media=disk,format=raw,file=fat:rw:disk"