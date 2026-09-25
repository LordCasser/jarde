## Why

合法 `putfield`/`putstatic` 可把 int 形状写入 `Z` 字段。jarde 已识别字段和写入，却因值没有 boolean 证明而引用该指令；整类仍能编译，拒绝方法成为空操作。在冻结的 380 行完整类中，B/C/S 285 行已正确，Z 95 行有 70 行错值、漏调用或漏异常，故编译成功不能作为恢复成功的判据。

## What Changes

- 仅在真实 `Z` 字段写入或已经验证为该写入的 accessor 消费位置，将可呈现的整数值按最低位表达为布尔值；已证明的 boolean/0/1 走现有路径。
- 用已有余数与比较表达式表示 `value % 2 != 0`，保留 receiver/value 一次求值、producer-before-null、真实 put 与 producer 来源。
- 固定 95 行 Z 的整类 JVM 行为对照；`ireturn Z`、boolean[] 的 `bastore`、普通赋值和调用参数继续独立处理。

## Capabilities

### Modified Capabilities

- `java8-recovery`: 在已证实的字段写入位置保留 JVM `Z` 的整数低位结果与副作用。

## Impact

局部修改 Java 恢复构建器的字段值消费分支；使用现有 AST/emit/source-map/预算，不新增依赖、IR pass 或类型求解器。保留现有 Class/reader 接口和通用位置转换合同。
