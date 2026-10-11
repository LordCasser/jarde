# classfile fixture census 增量复核

## 口径

读取 `crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections`：`class_fixture_paths()`（12243–12265）递归收集 `tests/fixtures` 下每个 `.class` 路径；每个 class 的 `header.methods` 逐个传给 `method_code_facts`（11829–11838），没有 Code 的方法只跳过。每个成功且完整解码的 Code body 令 `bodies += 1`（11839–11848）；handler 数是 `facts.exception_handlers.len()`（11848）；branch/switch 数是 `control_flow_targets()` 中非 `Handler` 的记录数（11844–11852）；subroutine 数按 opcode `jsr`/`jsr_w`（0xa8/0xc9）计数（11853–11861）。`control_flow_targets` 对普通 branch 每条写一项，对 switch 写 default 与每个 case 项，对 handler 另写 Handler 项（4428–4518），因此 handler 不进入 branch/switch 计数。

## 冻结 class 的解析结果

三份输入均为 major version 52（Java 8 classfile），取自已捕获的 javac 23 `--release 8` original 编译输出。用 pins 所列现有 Python classfile parser 的 instruction decoder 读取每个 Code body，并由 classfile 的 method/Code/exception-table 结构计数；switch 与 jsr 扫描均为零。类级增量如下：

| 新 class 路径 | Code bodies | 非 handler branch/switch target records | exception-table rows | jsr/jsr_w |
| --- | ---: | ---: | ---: | ---: |
| `cf11/UpstreamSequentialLoops2.class` | 3 | 10 | 0 | 0 |
| `cf11/ConditionalConstructionPositive.class` | 8 | 10 | 0 | 0 |
| `cf11/ConditionalConstructionRefusals.class` | 10 | 14 | 1 | 0 |
| 合计 | 21 | 34 | 1 | 0 |

Code methods共21个，其中 `UpstreamSequentialLoops2` 的 `<init>()V`、`asciiToLowerCase(Ljava/lang/String;)Ljava/lang/String;` 与 `<clinit>()V` 都有 Code。唯一 exception-table row 属于 `ConditionalConstructionRefusals.handlerAroundConditional(ZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;`，物理字段为 `start_pc=0, end_pc=24, handler_pc=25, catch_type=java/lang/IllegalArgumentException`。两份 accepted capture 的 Javap 与 classfile 都可独立确认此 row；没有把 handler 误算成 branch target。

上述新增文件分别增加 1 个 class path；不能按不同字节去重，因为测试遍历路径。把 `(1100, 4890, 468, 2953, 10)` 加上 `(3, 21, 1, 34, 0)` 得候选 `(1103, 4911, 469, 2987, 10)`。该数值是对新增三份 frozen class 的测量增量，尚未运行 corpus sweep。

## 使用边界

`classfile-census.patch` 仅修改 census 注释与 assertion tuple，不复制 class 文件，也不刷新 `corpus-fingerprint.json`。root 安装 fixtures 后须由现有 ignored updater 实际重算 fingerprint，再运行对应审计；本准备不声称最终 census 或 fingerprint 已通过。
