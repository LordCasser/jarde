# 字段、方法、参数声明注解：Java 8 完整类对照

`MemberTagged.java` 的一个字段、一个实例方法及其唯一参数都带 `@Deprecated`；`MemberRunner.java` 使用真实反射分别检查这三处，再调用普通方法作行为对照。runner 先把 `Field`/`Method` 存入 `Object`，在消费点作显式 cast：这是自写 Java 源码的一部分，保证 Jarde 现有数组元素静态类型局限不妨碍 runner 整类编译；不手工修改任何生成文本。

运行 `python3 openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/run_audit.py`。脚本用 `javac 23.0.1 --release 8 -g:none` 编译输入，记录两类的 class SHA 和完整 `javap -v -c -p`，经 JADX 1.5.6 与冻结 Jarde CLI（SHA-256 `f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544`）各生成**两类完整源码**。三套源码均独立通过 Java 8 编译和 `java -Xverify:all`；输出在 `generated/summary.json`：

| 类集 | 字段 `@Deprecated` | 方法 `@Deprecated` | 参数注解个数 | 普通方法值 |
| --- | --- | --- | --- | --- |
| 原 class | `true` | `true` | `1` | `5` |
| JADX 源码 | `true` | `true` | `1` | `5` |
| Jarde 源码 | `false` | `false` | `0` | `5` |

输入 `MemberTagged.class` 为 362B，SHA-256 `63fbb2c76199125f23540adc0611e65232be055e6817899291722c90d301c996`。`javap` 的三个 `RuntimeVisibleAnnotations`/`RuntimeVisibleParameterAnnotations` 属性分别在字段、方法和方法参数处；JADX 源码保留三处注解，Jarde 全部省略。原 class 的真实执行与重新编译后的 Jarde class 已证明差异，不能仅以生成源码无引用、javac 成功判断恢复。

`generated/` 留存原 class、完整 JADX/Jarde 源码、Jarde `--evidence all` JSON、javap、编译/运行日志与退出码。主代理复制到 `/tmp/jarde-member-ann-root-r0mtly_r/case` 后独立重放：`summary.json`、两份 Jarde 源码及三套运行输出均逐字节相同。属性层已具备成员 `AttributeShell`，类级注解内容读树实施中；成员宣告需额外处理字段/方法属性归属，参数属性还需按 descriptor 的参数**位置**映射，不能把 JVM slot 编号当成参数索引。当前证据仅显示缺口，不声称已经恢复；类级注解使用另案，不混入本项。
