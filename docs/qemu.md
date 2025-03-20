# 使用qemu-9.1.3的bug修复记录

## 错误描述
经过初步分析后，疑似qemu-9.1.3对loongarch的部分指令存在支持问题：
- ~~使用gdb调试到vldi指令时，会导致qemu卡住，原因不明~~

## 解决方法
1. 在`os/.cargo/config.toml`中添加`"-C", "target-feature=-lsx,-lasx"`
2. 使用被注释掉的print宏
3. ~~使用qemu-9.2.1版本~~
   - ~~qemu-9.2.1也无法执行vldi等指令~~
   - 可以通过控制rust工具链版本来使编译器不生成向量指令,尝试将rust-toolchain.toml的内容修改为channel = "nightly-2025-01-18"
   - 想要龙芯处理器支持向量指令集需要手动开启,故在entry.asm将euen寄存器写入值0x6,开启向量指令集支持