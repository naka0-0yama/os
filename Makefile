KERNEL = target/x86_64-unknown-none/debug/os
ISO = target/os.iso
ISO_DIR = target/isofiles
OVMF_CODE ?= OVMF/OVMF_CODE.fd
OVMF_VARS ?= OVMF/OVMF_VARS.fd

.PHONY: all build iso run clean

all: build

build:
	cargo build

iso: build
	@mkdir -p $(ISO_DIR)/boot/grub
	@cp $(KERNEL) $(ISO_DIR)/boot/kernel.bin
	@echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'menuentry "My Non-Paging OS" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '    multiboot2 /boot/kernel.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '    boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	@grub-mkrescue -o $(ISO) $(ISO_DIR)


img: build
	@echo 'insmod all_video' > grub.cfg
	@echo 'set timeout=0' >> grub.cfg
	@echo 'set default=0' >> grub.cfg
	@echo 'menuentry "My UEFI Multiboot2 OS" {' >> grub.cfg
	@echo '    multiboot2 /boot/kernel.bin' >> grub.cfg
	@echo '    boot' >> grub.cfg
	@echo '}' >> grub.cfg


	grub-mkstandalone \
		-O x86_64-efi \
		-o BOOTX64.EFI \
		"boot/grub/grub.cfg=grub.cfg" \
		"boot/kernel.bin=$(KERNEL)"
	
	@rm grub.cfg

	# 3. FAT32イメージ（ESP）を作成して BOOTX64.EFI だけを配置する
	rm -f os.img
	qemu-img create -f raw os.img 64M
	mkfs.vfat -n "MYOS" os.img
	mmd -i os.img ::/EFI
	mmd -i os.img ::/EFI/BOOT
	mcopy -i os.img BOOTX64.EFI ::/EFI/BOOT/BOOTX64.EFI
	rm BOOTX64.EFI

run: img
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive if=pflash,format=raw,file=$(OVMF_VARS) \
		-drive file=os.img,format=raw \
		-device qemu-xhci,id=xhci \
		-device usb-mouse,bus=xhci.0 \
		-machine q35 \
		-m 1G \
		-net none

run_nographic: img
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive if=pflash,format=raw,file=$(OVMF_VARS) \
		-drive file=os.img,format=raw \
		-device qemu-xhci,id=xhci \
		-device usb-mouse,bus=xhci.0 \
		-machine q35 \
		-m 1G \
		-serial stdio \
    	-display none
		-net none

clean:
	cargo clean