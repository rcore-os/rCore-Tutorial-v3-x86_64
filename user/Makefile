ARCH := x86_64
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(ARCH)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
ASMS := $(patsubst %, %.asm, $(ELFS))

BUILD_ARGS := --target $(ARCH).json -Zbuild-std=core,alloc,compiler_builtins
ifeq ($(MODE), release)
  BUILD_ARGS += --release
endif

OBJDUMP := rust-objdump --arch-name=$(ARCH) --print-imm-hex
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

all: build

elf:
	cargo build $(BUILD_ARGS)

$(ELFS): elf

$(TARGET_DIR)/%.asm: $(TARGET_DIR)/%
	$(OBJDUMP) -d $< > $@

disasm: $(ASMS) $(ELFS)

build: elf

clean:
	@cargo clean

.PHONY: all binary disasm build clean
