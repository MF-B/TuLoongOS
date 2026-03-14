TuLoongOS是一个将rCore操作系统移植至LoongArch64架构的操作系统项目

# 开发进度
- [X] 配置开发环境
- [X] 实现控制台输出与关机并添加Logger
- [X] 实现用户程序并添加基础系统调用
- [X] 实现特权级切换与批处理系统
- [X] 实现多道程序与协作式分时多任务
- [X] 实现进程管理（fork/exec/wait/kill）
- [X] 实现线程与同步机制（互斥锁/信号量/条件变量）
- [X] 实现虚拟内存管理（页表、地址空间与TLB相关支持）
- [X] 实现文件系统与IO抽象（easy-fs、pipe、stdio）
- [X] 实现块设备驱动（PCI + virtio-blk）

# 基础环境配置

## 实验环境

操作系统: Arch Linux x86_64

## 安装rust

```
curl https://sh.rustup.rs -sSf | sh
rustup install nightly
```

## 安装qemu模拟器
注意! 经测试，使用qemu-9.1.3版本时会出现bug，详细见[docs/qemu.md](docs/qemu.md)

```bash
# 安装编译所需的依赖包
echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm main" | sudo tee -a /etc/apt/sources.list
wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc
sudo apt-get update \
    && sudo apt-get install -y --no-install-recommends libclang-19-dev wget make python3 \
        xz-utils python3-venv ninja-build bzip2 meson cmake dosfstools build-essential \
        pkg-config libglib2.0-dev git libslirp-dev  \
    && sudo rm -rf /var/lib/apt/lists/*

# 安装与qemu相关的软件包
wget https://download.qemu.org/qemu-9.2.1.tar.xz
tar xvf qemu-9.2.1.tar.xz \
    && cd qemu-9.2.1 \
    && ./configure --prefix=/qemu-bin-9.2.1 \
        --target-list=loongarch64-softmmu,riscv64-softmmu,aarch64-softmmu,x86_64-softmmu \
        --enable-gcov --enable-debug --enable-slirp \
    && make -j$(nproc)
make install
# 配置环境变量(可自行添加进~/.bashrc中)
export PATH=$PATH:/qemu-bin-9.2.1/bin
# 测试是否正确安装
qemu-system-loongarch64 --version

rm -rf qemu-9.2.1 qemu-9.2.1.tar.xz
```

## 安装交叉编译工具
```bash
rustup target add loongarch64-unknown-none
```

# 使用
```bash
# 非调试运行
make run
# 调试运行
make debug
```
