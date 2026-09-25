# 非局部左值的后置递增返回值：架构边界

`CompoundProbe.java` 的 `return receiver().value++;` 与 `return data[index()]++;` 是普通 javac 生成的 Java 8 代码。原 class/JADX 的完整类编译执行七行全同；冻结 Jarde CLI 返回文本，但这两个方法没有返回语句，故完整类 javac 失败，不报告 Jarde 运行等价。class SHA-256、复跑命令和 stderr 在本目录 `README.md`/`run_audit.py`/`summary.json`。

| 方法 | 关键 BCI（javap） | 复制值的含义 | 当前结果 |
| --- | --- | --- | --- |
| `postField()I` | `receiver@0; dup@3; getfield@4; dup_x1@7; iconst_1@8; iadd@9; putfield@10; ireturn@13` | `dup_x1` 留下旧字段值，`iadd` 的新值写入同一实例字段 | 接收者调用被独立写出；复制/写/返回引用；方法缺 return |
| `postArray()I` | `data@0; index@3; dup2@6; iaload@7; dup_x2@8; iconst_1@9; iadd@10; iastore@11; ireturn@12` | `dup2` 保存数组与索引，`dup_x2` 留下旧元素值，`iadd` 的新值写入同一元素 | 索引调用被独立写出；复制/写/返回引用；方法缺 return |

现有 `build.rs::field_increments` 只接受八条相邻指令的 `load; dup; getfield; ...; putfield; ireturn`，要求接收者是可命名的局部变量，且只支持实例字段。它区分旧值/新值已有正确的 SSA 身份证明，但将完整 `this.n++` 或 `++this.n` 作为预格式化 `ExprKind::Local` 承载；这个技术债在加入调用接收者、数组下标、真实嵌套来源和括号时不能安全扩张。直接把原始接收者调用写成 `receiver(); return box.value++;` 也会写错目标实例，且没有保证异常/副作用次数。

候选后续改动应单独以 `recover-postfix-lvalue-values` 规划：复用字段/数组身份、SSA/复制与统一发射；只有在 `ireturn` 唯一消费复制出的旧值、同一基本块的写入存储计算后新值，且没有额外消费者时，呈现 `return receiver().value++;` 或 `return data[index()]++;`。目标可作为已有 `Field`/`Index` 表达式的**变量位置**，后缀运算需要一项真实表达式语义，不可把任意 `Field`/`Index` 读值当成可写左值。若引入节点，同一语法现存 `this.n++` 的预格式化文本应评估迁移，以免两套来源/括号规则长期并存；前缀 `++`、后置 `--`、局部存旧值或把旧值传参都需另测后决定是否同一最小闭环。

`recover-compound-lvalue-updates` 只处理写入语句没有复制结果留给调用方的 `+=`；它不应抢先认领上述 `dup_x1`/`dup_x2` 或吞掉旧值。Java 语言规范 [JLS 8 §15.14.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.14.2) 明确后置表达式的值为写入前的值，这就是必须以复制值的消费者而非相似 opcode 判定的关键。
