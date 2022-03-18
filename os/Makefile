# Building
ARCH := x86_64
TARGET := $(shell cat ../targets.txt | grep $(ARCH))
MODE := debug
KERNEL_ELF := target/$(ARCH)/$(MODE)/os

#BUILD_ARGS := --target $(TARGET) -Zbuild-std=core,alloc
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
	-device isa-debug-exit

GDB := gdb

build: kernel bootloader
	mkdir -p $(ESP)/EFI/rCore $(ESP)/EFI/Boot
	@cp ../rboot/target/x86_64-unknown-uefi/release/rboot.efi $(ESP)/EFI/Boot/BootX64.efi
	@cp ../rboot/rboot.conf $(ESP)/EFI/Boot/rboot.conf
	@cp $(KERNEL_ELF) $(ESP)/EFI/rCore/kernel.elf

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

bootloader:
	@cd ../rboot && make build

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

.PHONY: build env kernel clean disasm run run-inner debug
