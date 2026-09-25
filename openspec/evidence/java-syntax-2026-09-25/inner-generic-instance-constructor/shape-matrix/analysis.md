# JADX 内部类构造形状矩阵

[`replay.py`](replay.py) 对 `Outer` 和四个 caller 分别以 `javac --release 8 -g`、`-g:none` 编译，再用 `java -Xverify:all` 执行共同的 [`Runner`](fixture/Runner.java)。两种调试信息下运行轨迹相同：

```text
matrix.Outer$A$Plain:1
plain-null:1
matrix.Outer$A$Generic:2
generic-object-null:2
7:3
generic-typed-null:3
matrix.Outer$A$Plain:4
plain-raw-null:4
```

每个 caller 源文件又原样、独立地对仅含 `Outer` 类族的冻结目标 jar 编译；四个都通过。独立编译这些 caller 后再用原始 Runner 执行，轨迹仍与上面相同。生成的 `javap-g.txt` 和 `javap-g-none.txt` 固定 descriptor、`Signature`、`InnerClasses` 与调用顺序。`Plain` 构造器物理 descriptor 为 `(Lmatrix/Outer$A;I)V`；`Generic<V>` 的物理 descriptor 为 `(Lmatrix/Outer$A;Ljava/lang/Object;)V`，其泛型 `Signature` 为 `(TV;)V`。二者首个物理参数都是合成外部实例；泛型签名只描述源级参数。

JADX 1.5.6 对 `-g:none` 输入 jar 生成的 [`Outer.java`](jadx-source/Outer.java) 保留 `A<T>`、`Generic<V>` 声明、`V` 字段和访问器，构造器声明也只输出源级参数；该文件单独用 Java 8 编译通过。调用点形状则四种都丢失限定外部实例：

| Caller 形状 | JADX 调用表达式 | 对原始目标 jar 单独重编 |
| --- | --- | --- |
| 非泛型成员，参数化外层 | `new Outer.A.Plain(...)` | 缺少 `A.Plain` 的封闭实例 |
| 泛型成员，`Object` 返回 | `new Outer.A.Generic(...)` | 缺少 `A.Generic` 的封闭实例 |
| 泛型成员，参数化返回 | `new Outer.A.Generic<>(...)` | 缺少封闭实例；并报告原始类型上不能使用 diamond |
| 非泛型成员，raw 外层 | `new Outer.A.Plain(...)` | 缺少 `A.Plain` 的封闭实例 |

每个失败的完整诊断保存在 [`logs/`](logs/)，完整 JADX 输出连同所有 caller 的重编失败记录在 [`logs/jadx-full-source-javac.txt`](logs/jadx-full-source-javac.txt)。原始编译后的调用字节码在 `javap` 文件中先执行 `Objects.requireNonNull(outer)`，再执行 `mark(value)`，随后才调用携带外部实例首参的构造器。JADX 源也显式保留了空值检查在 `mark` 之前的顺序；由于原始生成源码编译失败，没有把它称作可执行行为对照，也没有从失败推断其他语义等价性。

矩阵说明故障跨越了 raw 与参数化外层、泛型与非泛型成员、`Object` 与参数化返回类型；这些变化都没有让 JADX 输出 `outer.new`。其中 `UseGenericTyped` 还在调用拼写中丢掉参数化外层，仅保留返回头的 `Outer.A<String>.Generic<Integer>`。这是对“调用点限定实例发射”形状的证据；`Outer.java` 的声明重编通过说明该样本的类声明可以独立编译，不证明反射签名或所有嵌套泛型组合均已恢复。

环境为 [`tool-versions.txt`](tool-versions.txt)。[`sha256.txt`](sha256.txt) 覆盖 fixture、脚本、分析、JADX 源码、javap 文本和诊断。脚本中所有 class、jar 及中间重编目录都位于 `TemporaryDirectory`，运行结束自动清理；它不运行 Cargo 或 Jarde。
