## Why

合法 Java 8 泛型覆写会同时携带源级 `String get()` 与编译器生成的 `ACC_BRIDGE Object get()`。Jarde 现有 `bridge@1` 已能恢复纯转发体，却在完整类源码中把两项都写成 Java 方法；冻结正例零引用而 javac 因同名同参拒绝，原 class 和同 JAR 输入的 JADX 类则编译运行一致。另一个合法、带副作用的桥接标志 class 证明无条件隐藏或改名也不忠实：JADX 改名后可编译，但 `get()Object` 的真实调用消失。

## What Changes

- 对已证明为编译器可重建的纯转发桥接方法，在类源码中只写源级覆写；物理方法表、原桥接方法的恢复报告、flags、调用目标及投影归属保持可审计。
- 桥接 flag、同次 `bridge@1` 转发证明、同类目标方法、已解析的继承契约及可重建的擦除签名必须共同成立。效果不纯、没有对应继承需求或任一事实缺失时明确拒绝投影，不以改名伪装为等价 Java 重载。
- 用普通泛型覆写与人工补丁的副作用桥接 class 比较原/JADX/Jarde 完整源码、编译、擦除调用与 `-Xverify:all` 执行。手删桥接声明的对照只证明方案可能性，不计作 Jarde 实现。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整类源码在能证明桥接成员由 Java 编译器从保留的覆写重新生成时投影该成员，并在证明不足时保留物理事实与明确拒绝。

## Impact

影响 `jarde-java::bridge@1` 的同次结构化交接、`src/facade.rs` 的类成员装配和 `src/class_source.rs` 的源码发射与投影归属。复用现有 classfile/SSA/桥接规则和按预算的环境读取，不新增 bridge 识别 pass、Java 源反解析、方法改名、泛型类型系统、依赖或运行目标代码。前置是已有 `bridge@1`、物理方法身份、类/继承元数据与 class-source 同次恢复路径；不顺带解决任意泛型 `Signature` 呈现、跨环境未知接口、注解/反射元数据重建或合成 accessor。证据见 `../../evidence/java-syntax-2026-09-23/bridge-source-projection/`。
