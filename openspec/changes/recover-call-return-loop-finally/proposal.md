# Proposal

## Why

原FX的`finReturn`在数组循环中先调用`c0(x)`保存返回值，再执行带StringBuilder与println的finally。当前完整生成类仍因missing return不能重编；独立同源码切片的两编译器控制、实际CFG/SSA探针已把缺口缩小到循环返回臂范围与三份多调用清理证明。

## What Changes

- 为已观察的固定数组循环形态建立私有、有界的调用返回finally证明，完整检查正常返回、循环后继与异常重抛三条路径，保持实际调用、构造器别名及异常优先级。
- 复用既有Finally/SavedReturn、受证书约束的循环范围授权与Tail，不扩展公共结构或通用清理语法。
- 固化独立原Java、未改driver、217B oracle及真实javac8/javac23 class，增加真实反例、停止、公共库/CLI与完整源码重编运行验收。
- 在最终产品自身CI通过后同步规格并归档；对原完整FX/FY/FO另存实际结果，保留其余失败。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在已证明的数组循环中恢复带调用返回及同一多调用finally的正常/异常路径。

## Impact

前提为既有只读真实IR、已实现的SavedReturn与私有loop_header授权；NoCall产品02de294f7af2d0798e33d52ce5474e60efe17b03的自身CI与主线交付另行完成，本片不得借用它的CI。预计修改`jarde-java`私有guard及测试、按需最小region接线、公共库/CLI测试和fixture指纹；不新增依赖、报告平面或预算维度。不是任意finally、任意循环、异常表或全部JAR/WAR的恢复承诺；不改原Java/driver/oracle/生成文本，不删除失败方法，不改旧全局计费基线。固定c0纯函数的运行不能直接观察调用次数/时点，必须另由SSA与来源闭合证明。
