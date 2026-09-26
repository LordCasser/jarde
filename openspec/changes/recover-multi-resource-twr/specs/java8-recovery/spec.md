## ADDED Requirements

### Requirement: Multi-resource try-with-resources proves its nested levels

两个及以上资源的 try-with-resources，在每层行的几何与每层的 close/`addSuppressed` 链都可证明时，SHALL 呈现为一条带多个资源的 `try` 头语句，资源顺序 = 初始化顺序。层级几何 MUST 从字节码证明：每层主异常表行从其资源初始化之后的受保护指令开始，止于该层正常 close 组的精确起点；内层 handler 的异常清理 MUST 由外层主行完整覆盖，或由范围恰好等于该清理段、且 catch 类型与目标 handler 均和外层主行相同的另一行覆盖。该证明 MUST NOT 单凭外层行是否止于内层 handler 的 span 末判定成功，也 MUST NOT 忽略任何额外保护行。任一层几何、close 顺序或抑制链不可证明时 MUST 保持整方法引用，不得部分呈现。

#### Scenario: Two resources present as one header

- **WHEN** 方法以两个资源打开、读取后返回，javac --release 8 编译（无 ifnull 资源头）
- **THEN** 恢复文本 SHALL 呈现 `try (<T> r1 = …; <T> r2 = …)` 一条语句；体内原先在 close 前求值、close 后纯读取并返回的值 SHALL 在该语句体内返回，重编译的 Java 8 源码 SHALL 不因局部作用域或返回位置而失败

#### Scenario: Both protection layouts remain provable

- **WHEN** 外层主行自身覆盖内层 handler 的完整清理段，或该段由一条同类型、同目标、范围精确相等的伴随行保护
- **THEN** 两种布局 SHALL 在其余证明相同的前提下呈现同一多资源语句；原有单行布局的两、三资源正例 MUST 保持可呈现

#### Scenario: Suppression order and exception types are preserved

- **WHEN** 受控执行对照在内层读取抛出异常、外层 close 抛出异常、双资源正常关闭三种输入下运行原 class 与重编译恢复文本
- **THEN** 返回值、主异常类型与 suppressed 异常的顺序 SHALL 一致

#### Scenario: Unprovable geometry keeps the quote

- **WHEN** 某层的行范围与正常 close 起点不能对上，内层异常清理缺少完整外层保护，或伴随行的类型、目标、范围任一不符（也包括资源初始化效果不在单一语句内、close 链缺失一级及返回尾部带独立效果）
- **THEN** 该方法 SHALL 保持引用并保留既有拒绝码，MUST NOT 输出丢掉一级 close 的 TWR 文本
