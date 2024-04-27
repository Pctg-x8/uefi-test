.PHONY: run

RUST_SOURCE_FILES=$(wildcard src/**/*.rs)
RUST_ARTIFACT_PATH=target/x86_64-unknown-uefi/debug/uefi-test.efi
BOOTLOADER_PATH=disk/EFI/BOOT/BOOTX64.efi
OVMF_PATH=Z:\OVMF.fd

$(BOOTLOADER_PATH): $(RUST_SOURCE_FILES) Cargo.toml
	cargo build --target x86_64-unknown-uefi
	cp $(RUST_ARTIFACT_PATH) $(BOOTLOADER_PATH)

run: $(BOOTLOADER_PATH)
	qemu-system-x86_64 -drive if=pflash,format=raw,file=$(OVMF_PATH) -drive if=ide,index=0,media=disk,format=raw,file=fat:rw:disk
