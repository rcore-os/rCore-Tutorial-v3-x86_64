# Building
ARCH := x86_64
MODE := debug
KERNEL_ELF := target/$(ARCH)/$(MODE)/os
FS_IMG := ../user/target/$(ARCH)/release/fs.img

BUILD_ARGS := -Z build-std=core,alloc,compiler_builtins --target $(ARCH).json
ifeq ($(MODE), release)
	BUILD_ARGS += --release
endif

# BOARD
BOARD ?= qemu

# Binutils
OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

# Disassembly
DISASM ?= -d

OVMF := ../rboot/OVMF.fd
ESP := target/$(ARCH)/$(MODE)/esp

# QEMU
QEMU := qemu-system-$(ARCH)
QEMU_ARGS := -nographic \
	-drive if=pflash,format=raw,readonly,file=$(OVMF) \
	-drive format=raw,file=fat:rw:$(ESP) \
	-serial mon:stdio \
	-m 4G \
	-device isa-debug-exit \
	-drive file=$(FS_IMG),if=none,format=raw,id=fsimg \
	-device ahci,id=ahci0 \
	-device ide-hd,drive=fsimg,bus=ahci0.0

GDB := gdb

build: kernel bootloader fs-img
	mkdir -p $(ESP)/EFI/rCore $(ESP)/EFI/Boot
	@cp ../rboot/target/x86_64-unknown-uefi/release/rboot.efi $(ESP)/EFI/Boot/BootX64.efi
	@cp ../rboot/rboot.conf $(ESP)/EFI/Boot/rboot.conf
	@cp $(KERNEL_ELF) $(ESP)/EFI/rCore/kernel.elf

bootloader:
	@cd ../rboot && make build

fs-img: $(APPS)
	@cd ../user && make build
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/$(ARCH)/release/

kernel:
	@cd ../user && make build
	@echo Arch: $(ARCH), Platform: $(BOARD)
	cargo build $(BUILD_ARGS)

clean:
	@cd ../user && make clean
	@cargo clean

disasm:
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

run: run-inner

run-inner: build
	$(QEMU) $(QEMU_ARGS)

debug: build
	@tmux new-session -d \
		"$(QEMU) $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "$(GDB) $(KERNEL_ELF) -ex 'target remote localhost:1234' -q -x gdbinit" && \
		tmux -2 attach-session -d

.PHONY: build kernel clean disasm run run-inner debug fs-img
