## Context

参见 [proposal.md](proposal.md) 与 [delta spec](specs/java8-recovery/spec.md)。已有零源参数常量体恢复依赖 enum 主类和选定匿名子类的同次 class facts，并且保留完整物理构造器关系；当前证明明确拒绝 `descriptor_source_argument_count != 0`。本提案要让一个源级 String 值穿过 javac 的 enum synthetic name/ordinal 构造层并与两匿名体形成原子投影。

## Goals / Non-Goals

**Goals:**

- 复用已有 enum-body 关系与候选路径，闭合 `TestEnums2a` 的双常量单 String 字面量形态。
- 在枚举常量源码、体内覆写方法与原始 class 来源之间建立一次性成功计划；只有该计划成功才能隐藏物理 bridge/子类记录。
- 缺事实、歧义、预算耗尽或停止时不泄露部分 enum-body 源投影。

**Non-Goals:**

- 不恢复普通带参枚举的 DT-11 行为，不处理 varargs。
- 不覆盖其它常量数量、任意参数类型或表达式、多个构造器、匿名类捕获、非标准编译器构造桥和不同的访问器变体。
- 不改变 reader 的公共属性模型、公开请求/API、通用匿名类恢复或目标类执行策略。

## Decisions

1. **通过既有类事实扩展局部成功证书。** 先复用主 enum、两项 `<clinit>` 常量构造引用、选定子类构造器、主类构造器与 synthetic 访问桥已有证据。只允许准确的两项有序常量；每项源参数必须由完整 Code 证明为单一 ASCII 字符串常量，且两项走同一 `(String,int,String)` 源构造器关系。若枚举的 `ACC_ABSTRACT` 由接口实现要求触发，本切片只接受恰好一个直接接口、该接口恰好声明一个 `public abstract` 方法且无 Code，并证明两个匿名体都实现同一签名；其它接口/抽象方法布局拒绝。`EnumCodeReference::String` 保留 modified UTF-8 原始字节，不能对其执行 lossy UTF-8 解码；该切片拒绝非 ASCII，除非现有路径能证明其可无损拼写。任何外加 synthetic 参数必须证明是纯桥接，name 与 ordinal 从入口原值传至枚举主构造器。备选方案是扩成任意数量、任意 Unicode 参数的解码与桥图解释器；这超出当前 fixture 证据，并扩大错误转义和错误内联风险。

2. **保持关系与输出原子。** 将字符串实参、两个常量与匿名体作为一个局部封闭计划，在现有 class-source 装配成功边界提交；计划失败时不先删掉物理声明再局部回退。继续使用现有完整类执行/预算状态，禁止在达到停止或输出预算后追加拒绝诊断或部分投影。备选方案是逐常量独立呈现；当第二个常量或 bridge 失败时，会得到不能表示原始物理关系的半枚举。

3. **复用现有常量体渲染，只加字符串构造实参。** 由已证明源字面量生成 enum 常量的 String 实参文本，匿名方法仍走已有 enum-body 输出路径。可与 DT-11 共享底层 raw Code `String` 引用解析和 Java literal spelling/escape 能力，避免复制 MUTF-8 到 Java literal 的实现；不得依赖 DT-11 普通 enum constant 的成功证书，因为匿名 owner 和桥关系不同，本切片必须独立原子合证。也不得通过文本搜索、class 名猜测或 JADX 的 `DONT_GENERATE` 决策判定 owner。备选方案是通用 Java AST 重写器，缺少跨类型能力且对这一局部形态引入不必要的实体。

4. **将 JADX 行为作为有界参考，而不是正确性证书。** `EnumVisitor` 从枚举 `<clinit>` 的构造调用识别子类 owner、跳过隐式 name/ordinal 并恢复余下实参；`ProcessAnonymous` 决定类是否可内联。项目的必要负例显示 JADX 在 verifier-valid 的 ordinal bridge 变化下会输出运行错误，因此 Jarde 必须验证桥接值和常量顺序，而非复刻它的标记策略。许可证允许参考，但它的 Dex visitor 不适配 Jarde 的 Rust classfile 证据和预算模型；不添加依赖。

## Risks / Trade-offs

- [classfile 字符串常量保存 MUTF-8 raw bytes，非 ASCII 到 Java literal 需要 UTF-16 解码和转义证明] → 首切片只接受 `"*"`、`"/"` 这类 ASCII 字面量；非 ASCII 字节不得 lossy decode，必须拒绝并保留来源证据。
- [合成访问桥可能含额外、看似无关的副作用或转换] → 验证方法目标、描述符、每个实参的值身份及返回/异常行为；必须证明纯转发才隐藏桥，否则保留物理事实。
- [输出预算或依赖遍历提前结束] → 将常量体及 bridge 证明置于同一计费和成功门内；预算/取消负例验证不生成半投影。
- [过拟合两个常量样例] → 将本次 contract 明确限为两个常量，未来拓展按独立证据和 OpenSpec 切片评估。

## Migration Plan

不涉及持久化格式或数据库迁移。实现只能增加有证据时的成功投影，拒绝路径继续保留物理结构；回滚时移除这一个证明切片及对应测试。
