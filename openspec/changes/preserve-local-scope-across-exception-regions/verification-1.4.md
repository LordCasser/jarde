## 1.4 catch 参数同槽绑定验收（2026-09-24）

输入均为 `javac --release 8 -g:none` 的冻结 Java 8 class：`Nest` SHA-256
`839e4803e97d5ed1768dddd255ccbba794f53a0b82c4b0cc1e980f8b329a83b0`，
`TypedCatch` SHA-256 `efc70f670fa3577c42abeaa026e8b194d196a0839578add4fa9e30311b94f4a0`，
`Stated` SHA-256 `a7e449e308b1622c6eefa5f09b482d6a397ec0de3d717bad9183c61ab319e1ad`。

改动前定向测试的真实拒绝：`Nest.nest` 为 `local 1 escapes catch parameter scope at region [0, 0, 1]`；
`TypedCatch.twoCatches` 与 `Stated.steps` 均为 `local 1 escapes catch parameter scope at region [0, 1]`。
三个输出当时均含 `@bytecode`，没有目标 catch 正文。`Stated.steps` 是两个顺序 try/catch，
无分支或循环，故此错误不属于 loop transfer。

改动后按 handler entry store 和 SSA 的 phi、局部 load/store 值链检查 clause 外读取；
三个目标方法均恢复 catch，定向测试通过。`Nest`、`Stated` 的 Jarde **完整类**
用 `javac --release 8` 编译，原 class、JADX 1.5.6 完整类及 Jarde 完整类
均通过 `java -Xverify:all`，同一 driver 的输出依次为 `0,-1,-2,8,-2,4`。
`TypedCatch.twoCatches(-1,0,11)` 的原 class、JADX 完整类与只保留已恢复成员的
Jarde 类均输出 `-1,0,-2`。Jarde `TypedCatch` 完整类仍因独立的
`finallyIncrements` quoted fallback 缺少返回语句而不能编译；此处不将成员投影称作完整类验收。

反向边界使用冻结 `CatchEscape.class` 的 handler 序列，把 `astore_2; aload_2`
定长改成 `astore_1; aload_1`，让 handler entry 值经一次赋值在 catch 外读取。
补丁后 class SHA-256 为 `c2d2a05944fce7fdaca200800191df30754f7d3de5ec32a6a6aad1646c3726fd`，
`java -Xverify:all` 执行普通/异常输入返回原 action 对象类与 `IllegalArgumentException`。
Jarde 完整拒绝并覆盖 `@bytecode 0 11 14`，诊断为
`local 1 escapes catch parameter scope at region [0, 1]`；不输出 catch 外 `return local1`。

未归因项：`p3_typed_catch::a_catch_type_of_zero_becomes_neither_a_catch_nor_a_finally`
仍报 `local 1 crosses a quoted fallback region`，由任务 2.5 另行审计。本文不为
1.2、1.3、1.5、2.1–2.5 提供完成证据。

root 独立用清理前保留的同版 CLI（SHA-256
`95362354d3de2af1ac57f69ea9f7492731ca580afd2e5f7e3b16fc4144b3aa8c`）
重放三份冻结输入，直接检查完整类正文。`Nest` 和 `Stated` 原样复制为对应
`Nest.java`、`Stated.java`，以 Java 8 重编；同一 driver 的原 class 与 Jarde
输出都通过 `-Xverify:all`，六行逐字相同：`0,-1,-2,8,-2,4`。root 从冻结
`CatchEscape.class` 唯一的五字节 handler 序列独立生成反例，SHA-256 与上文
`c2d2a059...` 相同，CLI 仍指名 `@bytecode 0 11 14` 及 catch 外读取拒绝。
