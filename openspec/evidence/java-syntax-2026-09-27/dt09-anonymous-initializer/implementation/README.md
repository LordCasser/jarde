# DT-09 实现后重放

本目录由同级 `replay.py --expect-projection` 生成。命令会从 `input/p` 编译原始 class，使用固定 JADX 导出完整源码，再从同一 JAR 用 Jarde 导出根类与物理 child；三套完整源码均以 `javac --release 8` 编译，并由同一个 Runner 在 `java -Xverify:all` 下执行。

从仓库根目录重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt09-anonymous-initializer/replay.py \
  --jarde /path/to/jarde-cli \
  --jadx /path/to/fixed-jadx \
  --out /tmp/dt09-anonymous-initializer-implementation \
  --expect-projection
```

`summary.json` 记录固定 JADX revision、输入与生成源码 SHA-256、Jarde CLI SHA-256、编译/运行退出状态和输出。三者都输出 `1\n8`；JADX 与 Jarde 都先在匿名体初始化块中写入 `1`，再进入 `run()` 覆盖方法。

本验收只覆盖直接返回、同包可拼写的普通父类、无捕获且无字段的匿名 child，以及准确 `super()` 后单次根类静态 `int` 写入。JDK `Thread`、外层私有实例字段和非返回分配仍是独立边界。
