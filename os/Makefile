TARGET := target/loongarch64-unknown-none/release/os
QEMU := qemu-system-loongarch64
LOG ?= INFO
USER_DIR := ../user

QEMU_ARGS := -m 1G -smp 1 -machine virt -nographic -kernel $(TARGET)

# 默认目标
all: build

# 构建项目
build:
	cargo build --release

# 构建用户程序
build-user:
	@echo "Building user applications..."
	@cd $(USER_DIR) && make build

# 运行项目
run: build-user build
	$(QEMU) -m 1G -smp 1 -machine virt \
		-nographic \
		-kernel $(TARGET)
		

# 调试模式运行
debug: build-user build
	@tmux new-session -d \
		"$(QEMU) $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "~/osEnv/gdb/bin/loongarch64-unknown-linux-gnu-gdb -ex 'file target/loongarch64-unknown-none/release/os' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

# debug: qemu-version-check build
# 	@tmux new-session -d \
# 		"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
# 		tmux split-window -h "~/osEnv/gdb/bin/loongarch64-unknown-linux-gnu-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
# 		tmux -2 attach-session -d

# 清理用户程序
clean-user:
	@echo "Cleaning user applications..."
	@cd $(USER_DIR) && make clean || true

# 清理项目
clean: clean-user
	cargo clean

.PHONY: all build run debug clean