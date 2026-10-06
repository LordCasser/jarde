## ADDED Requirements

### Requirement: 后缀自增旧值的条件位呈现

当 `iinc` 的旧值 load 跨 `iinc` 被循环或分支测试读取（单变量、单条件位）时，系统 SHALL 将测试表达式按源码形态呈现（如 `while (arg0[arg1++] != 0)`），方法行为完整。

#### Scenario: do-while 扫描主锚
- **WHEN** 输入为固定巡查 fixture 的 do-while 扫描形（`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且剥离编译后 `-Xverify:all` 输出与原一致（迭代次数精确）

#### Scenario: A 相锚零回退
- **WHEN** 输入为 `recover_postfix_old_value_snapshot` 的 A 相锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 复合与陷阱形仍拒
- **WHEN** 多变量复合条件、短路链中段或 `i = i++` 陷阱
- **THEN** 拒绝 SHALL 逐字保持
