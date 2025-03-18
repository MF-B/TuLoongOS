TARGET := target/loongarch64-unknown-none/release/os
QEMU := qemu-system-loongarch64
LOG ?= INFO
USER_DIR := ../user

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
	$(QEMU) -m 1G -smp 1 -machine virt \
		-nographic \
		-kernel $(TARGET) \
		-s -S

# 清理用户程序
clean-user:
	@echo "Cleaning user applications..."
	@cd $(USER_DIR) && make clean || true

# 清理项目
clean: clean-user
	cargo clean

.PHONY: all build run debug clean