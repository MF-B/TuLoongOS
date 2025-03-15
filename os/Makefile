TARGET := target/loongarch64-unknown-none/release/os
QEMU := qemu-system-loongarch64
LOG ?= INFO

# 默认目标
all: build

# 构建项目
build:
	cargo build --release

# 运行项目
run: build
	$(QEMU) -m 1G -smp 1 -machine virt \
		-nographic \
		-kernel $(TARGET)
		

# 调试模式运行
debug: build
	$(QEMU) -m 1G -smp 1 -machine virt \
		-nographic \
		-kernel $(TARGET) \
		-s -S

# 清理项目
clean:
	cargo clean

.PHONY: all build run debug clean