# jadx vs jarde：正确性与可读性对照（本轮实测记录，2026-09-21）

环境：jadx 1.5.6（`/opt/homebrew/bin/jadx`）+ OpenJDK 23.0.1；jarde 本工作区 debug/release 构建。
语料：`/Users/lordcasser/workspace/vulnerability/vulhub`（12 artifact，哈希与 `openspec/benchmark-protocol.md` 一致，本轮全部 12 项已复核）。
**本文只记录口径与事实，不给加速倍数**；正式计时见 `raw.jsonl` 与本轮报告。

## 1. 产出单位不同（先写清楚，避免误读）

| 工具 | 单位 | 内容 |
| --- | --- | --- |
| jadx | 整个 artifact（或 `--single-class` 一类） | 整类 Java 文本，含 package/import、注解、`throws`、内部类内联；**省略隐式 `<init>`、把 `<clinit>` 折进字段初始化** |
| jarde `export` | 显式物理范围（默认 `snapshot_all`，可给 `artifact_tree`） | 逐方法 JSONL：方法身份 + `RecoveryReport`（文本、source map、规则、拒绝、质量/覆盖/执行平面） |
| jarde `class-source` | 一个类 | 装配文本：类声明 + 字段 + 每个成员签名与体，按位置标记未产出/拒绝/仅解释/无 Body |

结论要按单位写：`jadx 整 artifact` vs `jarde 显式 scope 全量`是两个不同产出量；方法集合要先对账（jadx 折叠/省略的成员单列）。

## 2. 可读性并排样本

### 2.1 `org.bouncycastle.asn1.ASN1Object`（bcprov，无本地变量表）

jadx：

```java
package org.bouncycastle.asn1;
import java.io.ByteArrayOutputStream;
…
public abstract class ASN1Object implements ASN1Encodable, Encodable {
    @Override // org.bouncycastle.util.Encodable
    public byte[] getEncoded() throws IOException {
        ByteArrayOutputStream byteArrayOutputStream = new ByteArrayOutputStream();
        new ASN1OutputStream(byteArrayOutputStream).writeObject(this);
        return byteArrayOutputStream.toByteArray();
    }
```

jarde `class-source`：

```java
// jarde: presentation of `org/bouncycastle/asn1/ASN1Object` …
public abstract class org.bouncycastle.asn1.ASN1Object extends java.lang.Object
        implements org.bouncycastle.asn1.ASN1Encodable, org.bouncycastle.util.Encodable {
    public byte[] getEncoded() {
        // @method getEncoded()[B
        java.io.ByteArrayOutputStream local1 = new java.io.ByteArrayOutputStream();
        org.bouncycastle.asn1.ASN1OutputStream local2 = new org.bouncycastle.asn1.ASN1OutputStream(local1);
        local2.writeObject(arg0);
        return local1.toByteArray();
    }
```

**同**：语句序列、对象构造与调用、返回表达式与 jadx 一致（同一字节码事实）。

**jadx 更可读之处**：package/import 让类型名短；`this` 代替 `arg0`；参数/局部名可读（`byteArrayOutputStream`）；保留 `@Override`/`throws`。

**jarde 更可读（或更诚实）之处**：`// @method`/`// @declaration` 让每个成员的身份与成员标志可见；`local1/local2` 与 `arg0` 是"确定性的序数名"，不冒充源码名；每个成员带 source map，可回到 BCI。

### 2.2 `org.bouncycastle.asn1.ASN1OutputStream.writeLength(int)`（有控制流）

jadx 写出实际代码（`if (i <= 127) { write((byte) i); return; } int i2 = 1; int i3 = i; …`），并重建了内部类 `ImplicitOutputStream`。

jarde 对同一方法**部分拒绝**：

```java
    void writeLength(int arg1) {
        // @method writeLength(I)V
        if (arg1 > 127) {
            int local2 = 1;
            int local3 = arg1;
        } else {
            // @bytecode 68
            // the instruction at BCI 68 is not part of the provable subset
            // @bytecode 69
            // the value at BCI 69 comes from an Other at BCI 68, which produces no expression this subset writes
        }
        return;
        // @bytecode 10 19 25 43 48 63
        // 6 live block(s) are reachable only through edges the normal-flow view leaves out: [10, 19, 25, 43, 48, 63]
    }
```

这是本轮最重要的一条对照事实：**jadx 产出更多代码，jarde 在无法证明的地方写出 BCI 与原因而不是猜**。两者不是同一质量口径下的"谁快"，而是"谁肯写不确定的代码"。任何"jarde 慢/弱"的结论都必须先说明产出量差异。

### 2.3 实例接收者拼写

- jadx：`this.empId`、`this.first = true`。
- jarde：有 `LocalVariableTable` 时 `this_.empId`（`this` 是关键字 → 别名为 `this_`），无该表时 `arg0.os`。

已确认为缺陷并立项：`openspec/changes/spell-the-instance-receiver-as-this`（证据含本样本与 `EmployeeAction` 的 7 处 `this_`）。

### 2.4 jarde 独有

每方法 source map（BCI ↔ 文本区间）、规则列表、拒绝记录、`content`/`quality`/`coverage`/`execution` 四平面；
jadx 不提供这些。

## 3. 正确性对照口径（本轮执行的部分与未执行的部分）

**已执行（本轮）**：
- 库级：`tests/p3_prepared_input.rs` 逐字段证明 prepared 输入与直接路径产出同一报告与同一文本；`tests/bulk_recovery_workers.rs` 证明 1 与 N worker 的逐方法文本/身份/顺序一致；`tests/class_source.rs` 证明装配文本在读取形状改动前后逐字节相同。
- CLI 级：`crates/jarde-cli/tests/export_cli.rs` 逐记录比对"进程内 `Engine::recover_all` + 记录 sink"与真实 JSONL（除 elapsed 整条相等）。
- 行为级（复用 benchmark session 的受控 JDK 编译执行 harness，`/tmp/jarde-bench/correctness-head/`）：该 harness 的 jarde 侧输入是旧 sweep JSONL 形状；本轮 `export` 是新的记录形状，**未适配**，因此本轮没有重跑"恢复文本编译执行与原类逐 trace 比较"。旧结论（8807fa5：抽样 1,157 → 可比 818 → 817 一致）对应的引擎版本早于本轮的 prepared/批量接线。

**未执行（如实列出）**：
- 用本轮 `export` 输出重跑受控执行对照（需要把 harness 的输入读到新记录形状）；
- jadx 侧同批方法的执行对照（旧结论为 45/45，样本更小）。

## 4. 由本轮对照/实现发现的问题（已按 OpenSpec 立项或已修）

| 问题 | 落点 |
| --- | --- |
| 接收者被拼成 `this_`/`arg0` | 新变更 `spell-the-instance-receiver-as-this`（0/7） |
| `class-source` 每成员各读一次类头 | 本 change 任务 7.3：改为消费一次 prepared 类（`class_headers` 常数 2，文本不变） |
| 整包导出每类重解析容器目录、默认额度下跑不完 | 本 change 任务 5.4（实现中） |
| sink 拿不到总账许可（输出额度两本账） | 本 change 任务 4.6（未实现） |
| `ClassMemberFacts::method_count` 在字段表截断时为 0，coverage 范围显示"0 of 0" | 记录待决（本轮不改；锚点 `crates/jarde-reader/src/classfile.rs::class_member_facts`、`src/facade.rs::member_coverage`） |
