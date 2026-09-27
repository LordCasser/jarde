# CF-03 区域负例夹具

`cf03/BranchShapes.class` 由固定的 `openspec/evidence/java-syntax-2026-09-27/cf03-branches/input/cf03/BranchShapes.java` 以 `javac --release 8 -g:none` 编译，SHA-256 为 `c970900d817ae0a6f77cebb06887868e8d89864a5a9fbe00541b514312c998bf`。

两个负例先用相同的 Java 8 命令编译同目录源码，再只改方法的 Code 字节。`Cf03Negative.class` 将 `extra` 的 `1a 99 00 05 03 ac 1b 99` 改成 `1a 9a 00 1d 00 00 1b 99`：BCI 0 可以绕过 BCI 6 进入 BCI 30，故共同尾有外部前驱。`Cf03Ambiguous.class` 将 `f` 的 `1a 99 00 12 1b 9a 00 06 10 64 ac 1c 99 00 16 11 00 c8 ac` 改成 `1a 99 00 12 1b 99 00 12 a7 00 03 1c 99 00 16 a7 00 0f 00`，再通过 JDK 内置 ASM `ClassReader.SKIP_FRAMES` 与 `ClassWriter.COMPUTE_FRAMES | COMPUTE_MAXS` 重算 stack maps：BCI 4/19 都能到 BCI 23、30、34，其中 23 与 30 互不可达，不能任选一个共享尾。两个修改后的类均已用 `java -Xverify:all` 加载并运行；源码是修改前的编译输入，物理控制流以所存 `.class` 为准。
