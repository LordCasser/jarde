# DT-29 同包直接父类字段写入

这个夹具只覆盖 `B extends A` 中对 `A` 所声明的 `protected` 和包可见实例字段写入。`B` 各自声明了同名字段；Runner 反射读取 A/B 四个字段，只有 A 的两个字段应变为 `true`。`B.set` 还保留既有 public 字段和 Java 8 private accessor 控制。

回放脚本用 `javac --release 8 -g:none` 编译输入、固定 JADX checkout 和 Jarde 输出的完整类族，并用 `java -Xverify:all` 运行三组源码。三方均打印 `true:true:false:false`。输入 `javap` 记录确认 protected/package-private 写入分别位于 `B.set` 的 BCI 7/12，CP owner 均为 `A`；Jarde 报告在相同 BCI 保留来源，并输出显式 `((A) this)` owner cast。正例完整类族没有 `@bytecode` 拒绝片段。

回放还逐项检查访问 flags、owner/name/descriptor、直接父类关系、实际 SSA 接收者、跨包 protected 源码合法性、public-final 既有路径、private accessor 和方法体预算停止。拒绝项继续保留对应 BCI。跨包控制的 Java 原始源码可编译；把接收者改写成 `((p.A) this)` 会被 javac 拒绝，因此跨包情形不发本证书。

回放还只针对固定 DT-29 组合的 `FieldCast$B.self(Z)V` 重检 BCI 7/12：两项均以 `FieldCast$A` 为物理 CP owner，结构化正文与来源映射均保留这两个 BCI。`C`、`D` 及 root 的拼接/run/bits 问题继续作为独立范围留存。

需要先在仓库根目录构建 CLI，再设置 `JARDE_CLI` 重放：

```sh
CARGO_TARGET_DIR=/tmp/jarde-same-package-parent-target cargo build --locked -p jarde-cli
JARDE_CLI=/tmp/jarde-same-package-parent-target/debug/jarde-cli \
  python3 openspec/evidence/java-syntax-2026-09-27/dt29-same-package-parent-field-writes/replay.py
```

固定 JADX checkout 为 `/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。三方源码、反射 Runner、`javap`、方法报告、运行结果和 SHA-256 清单保存在 `outputs/`。`javap` 中的临时目录及 JSON 报告中的墙钟耗时会在写入证据前规范化，重放结果可稳定比较。
