# 加载地址
- 0x0 - 0x0fffffff都是可以加载的,但0x0 - 0x200000存在qemu的bios和其他文件,故将内核加载到0x200000,
经查看内核大小以及各段地址,暂时将应用程序加载到0x800000区域.
- 0x80000000 - 0xafffffff 这段空间也是可用的

# issue
- user/build.py会自动把BASE_ADDRESS前面的0抹去,所以当BASE_ADDRESS前面有0的时候,需要将user/linker.rs前面的0也抹去,否则匹配不到