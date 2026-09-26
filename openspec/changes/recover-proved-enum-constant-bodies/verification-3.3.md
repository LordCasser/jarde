# 3.3 类级投影失败与来源验收

2026-09-26，root 审读类级投影的局部 `constants_text` 装配及 `None`/预算停止路径；新增的子类物理身份注释只在整组投影成功时进入类源码。第二个带体常量的方法文本失效时，投影整体拒绝，主枚举原字段和构造器、子类独立方法 Code 均保留。JSON 字段/方法身份与来源映射不依赖投影，all/essential 产生相同的类源码。输出、读取、IR 紧预算及预先取消不会留下半个常量体。

root 独立运行 `CARGO_TARGET_DIR=/tmp/jarde-enum-root-target cargo test -p jarde --lib --locked`：75/75 通过，其中三个新增投影测试通过；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constant-bodies --strict` 均通过。4.1 的完整三方验收仍单列，不能以本步骤代替。
