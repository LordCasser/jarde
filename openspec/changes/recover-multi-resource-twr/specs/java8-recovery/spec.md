## ADDED Requirements

### Requirement: Multi-resource try-with-resources proves its nested levels

两个及以上资源的 try-with-resources，在每层行的几何与每层的 close/`addSuppressed` 链都可证明时，SHALL 呈现为一条带多个资源的 `try` 头语句，资源顺序 = 初始化顺序。层级几何 MUST 从字节码证明：每层的异常表行从该层资源初始化前开始，止于该层语句自身的结束处（内层资源声明、体与内层正常 close 链之后）；该证明 MUST NOT 要求外层行止于内层 handler 的 span 末。任一层几何、close 顺序或抑制链不可证明时 MUST 保持整方法引用，不得部分呈现。

#### Scenario: Two resources present as one header

- **WHEN** 方法以两个资源打开、读取后返回，javac --release 8 编译（无 ifnull 资源头）
- **THEN** 恢复文本 SHALL 呈现 `try (<T> r1 = …; <T> r2 = …)` 一条语句，体为读取与返回；不再整方法字节码引用

#### Scenario: Suppression order and exception types are preserved

- **WHEN** 受控执行对照在内层读取抛出异常、外层 close 抛出异常、双资源正常关闭三种输入下运行原 class 与重编译恢复文本
- **THEN** 返回值、主异常类型与 suppressed 异常的顺序 SHALL 一致

#### Scenario: Unprovable geometry keeps the quote

- **WHEN** 某层的行范围与该层语句的边界不能对上（例如资源初始化效果不在单一语句内，或正常 close 链缺失一级）
- **THEN** 该方法 SHALL 保持引用并保留既有拒绝码，MUST NOT 输出丢掉一级 close 的 TWR 文本
