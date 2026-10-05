## Why

[unicode-identifier 巡查](../../evidence/java-syntax-2026-10-05/unicode-identifier-patrol/README.md)：CJK 标识符（`变量`/`描述`/`方法`）在**声明层**被 `is_java_identifier`（names.rs:125，ASCII-only：`is_ascii_alphabetic`）判为非标识符，替换为 `__`——两个中文字段同名碰撞、渲染不可编译；方法体内同一名字的引用却正确呈现（双源不一致）。手工还原 3 处 `__` 后全链编译并行为一致，证明唯一根因是该判定。呈现缺陷级（与响亮拒绝同级严重性：字段名错导致后续引用全部失配）。

## What Changes

- `is_java_identifier` 的字符判定从 ASCII-only 扩为 **JLS Character.isJavaIdentifierStart/Part 语义**（Java 语义超集：CJK/Unicode 字母/数字；保持关键字排除与 `_$` 规则不变）。
- 声明层与引用层用同一判定（消除双源不一致）。
- raw name 已是合法池 UTF-8 字节（JVMS 4.7.7），无需新解码。

## 硬不变量

1. 现有 ASCII 标识符路径渲染逐字节不变（corpus fingerprint 零回退）。
2. 不新增机制（判定函数内扩展）。
3. 池形 `$` 合成名（`V1$Op` 等 names.rs:162 家族）不受影响。

## 验收

- UT.java fixture（`--encoding UTF-8 --release 8`）：字段/方法声明呈现中文名、无 `__` 替换注释、渲染剥离编译 exit 0、行为 `变量=1/42/中文` 一致。
- 全门禁 + fixture 双协议。
