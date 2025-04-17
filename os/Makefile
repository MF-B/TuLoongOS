# Building
TARGET := loongarch64-unknown-none
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=loongarch64
OBJCOPY := rust-objcopy --binary-architecture=loongarch64

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

build: env $(KERNEL_BIN) fs-img 

env:
	(rustup target list | grep "loongarch64-unknown-none (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/loongarch64-unknown-none/release/

$(APPS):

kernel:
	@echo Platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build --release
	@rm src/linker.ld

clean:
	@cargo clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: run-inner

QEMU_ARGS := -machine virt \
			-m 1G -smp 1 \
			-nographic \
			-kernel $(KERNEL_ELF) \
			-device virtio-blk-pci,drive=disk0 \
			-drive id=disk0,if=none,format=raw,file=$(FS_IMG)

QEMU_NAME := qemu-system-loongarch64
qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU_NAME)

run-inner: qemu-version-check build
	@qemu-system-loongarch64 $(QEMU_ARGS)

debug: qemu-version-check build
	@tmux new-session -d \
		"qemu-system-loongarch64 $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "~/osEnv/gdb/bin/loongarch64-unknown-linux-gnu-gdb -ex 'file $(KERNEL_ELF)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	@qemu-system-loongarch64 $(QEMU_ARGS) -s -S

gdbclient:
	@loongarch64-unknown-linux-gnu-gdb -ex 'file $(KERNEL_ELF)' -ex 'target remote localhost:1234'

.PHONY: build env kernel clean disasm disasm-vim run-inner fs-img gdbserver gdbclient qemu-version-check