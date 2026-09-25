# 擦除返回与 instantiated 返回对照

`ReturnProbe.genericString()` 返回 `Supplier<String>`，implementation handle 是泛型 `<T> T ReturnHelper.value()`，其 JVM descriptor 为 `()Object`。擦除 SAM 也是 `()Object`，instantiated SAM 是 `()String`。

runner 分别以 raw 与 typed 路径调用同一个函数对象。raw `Supplier.get()` 返回真实对象；Integer 状态输出 `raw:java.lang.Integer:7`。typed 路径调用后由调用者执行 `checkcast String`，同一 Integer 状态输出 `typed:java.lang.ClassCastException`。`runner-javap.txt` 固定这个 checkcast 位于调用者的 `get()` 之后；函数体没有新增返回检查。

重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/generic-method-references/expanded/return-probe/run_probe.py
```

原 class 为 821 bytes、2 个 methods/Code attributes、1 个 bootstrap entry，SHA-256 为 `adfbe98a1409fd4b7cde9020ad66633f3f4bd2d3227e634d495721b81d1c1f3c`。原、JADX 和 Jarde 完整类在 String/Integer 两种输入下编译执行相同；完整源、javap、编译/JVM 状态、输出、输入 hash 与 CLI hash 均保存在本目录。该用例是通过对照，不能因 `instantiated SAM` 返回窄于 erased SAM 而降为拒绝或在函数体添加 checkcast。
