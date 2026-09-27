## 1. 固定完整源单元和有效反例

- [x] 1.1 冻结 `Generic.class`/`Generic$A.class` 哈希、双向关系、class/field/method Signature、桥 Code；重放原/JADX 完整 `multi` Java 8 源码、原 API Runner 与桥 consumer。
- [x] 1.2 制作 verifier 有效的错 self row、接口表/Signature 不同、未知 `T`、桥目标/cast/额外效果、根额外使用与预算/取消近邻；记录物理事实和可观察行为。

## 2. 同一源单元的联合证明

- [x] 2.1 将唯一静态成员声明关系与构造使用点分离，仍核选定定义、双向关系、完整成员、根使用闭包；既有构造型/Shape pair 路径不回归。
- [x] 2.2 在 root family 词法上下文中用 reader Signature parser/erasure proof 核 child 类头、字段 `T`、typed 方法 `A<T>` 与真实 `Comparable` 接口，保留物理 child 原样报告。
- [x] 2.3 证明 `compareTo(Object)` 的 flags、参数 cast、一次 typed 转发、返回和异常表与受证泛型声明的编译器桥一致；异常或错桥拒绝，不能放宽独立 `bridge@1`。

## 3. 原子源码与来源

- [x] 3.1 在已有 member-family writer 中一次输出完整 `A<T>`、字段、构造器、typed 方法并省略已证桥；无半份投影、无 `$` 名猜测，各派生段具有物理锚点。
- [x] 3.2 覆盖阶段预算、输出预算、取消及物理报告保留；拒绝时根源码维持准确物理基线。

## 4. 三方验收

- [x] 4.1 fresh CLI 原/JADX/Jarde 完整 `multi` Java 8 源码及外部 consumer 重编，`java -Xverify:all` 运行、泛型反射、桥反射和错误参数异常一致；1.2 全部近邻拒绝。
- [x] 4.2 运行 Shape pair、单静态成员、泛型签名/桥及 class-source 回归，workspace check、fmt、OpenSpec strict、diff check；清理专用 Cargo target。
- [x] 4.3 root 独立核关系、Signature 作用域、桥再生与完整三方运行，更新 EM-01 清单；只标记固定 Generic.A 子形态。
