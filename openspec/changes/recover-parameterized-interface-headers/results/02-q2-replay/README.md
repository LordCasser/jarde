# 任务 1.4：池形（`$`）拼写的判别实验复验 + 接口位复验

root 已在 `/tmp/rq2b` / `/tmp/rq2d` 做过判别实验（design 的两段 root 更正）。本片按 tasks 1.4
以**冻结脚本**重做入证据目录，并加两格：深嵌套 `D$Mid$Leaf<Integer>`（tasks 明列）与**接口位**
（`implements DI$Inner<String>`，本片新增，为 2.1 的 javac 腿）。

```sh
sh results/02-q2-replay/q2-replay.sh      # 输出：q2-replay.out
sh results/02-q2-replay/iface-position.sh # 输出：iface-position.out
```

## 结果（`q2-replay.out` / `iface-position.out` 逐字）

| 格 | javac 8（Corretto 1.8.0_432） | javac 23.0.1 `--release 8` | 池形 vs source 形产物 |
| --- | --- | --- | --- |
| `extends D$Base` / `D.Base`（裸） | exit 0 / 0 | exit 0 / 0 | `cmp` **逐字节相同**（两腿） |
| `extends D$Base<String>` / `D.Base<String>` | exit 0 / 0 | exit 0 / 0 | `cmp` **逐字节相同**（两腿） |
| `extends D$Mid$Leaf<Integer>` / `D.Mid.Leaf<Integer>`（深嵌套） | exit 0 / 0 | exit 0 / 0 | `cmp` **逐字节相同**（两腿） |
| `implements DI$Inner<String>` / `DI.Inner<String>`（接口位） | exit 0 / 0 | exit 0 / 0 | `cmp` **逐字节相同**（两腿） |
| `class Self$Name implements java.lang.Comparable<Self$Name>`（`BR$Impl` 形） | exit 0 | exit 0 | javac 自建擦除桥（`javap`：`ACC_BRIDGE, ACC_SYNTHETIC`，转发 `compareTo:(LSelf$Name;)I`） |
| 负例自检 `extends D$NotThere<String>` | **exit≠0** | **exit≠0** | —（脚本要求"能失败才可信"） |

**本片把 root 的"superclass 项 + InnerClasses 行恒等"加强为整文件 `cmp` 恒等**（同名同类、只差父类
拼写的两格），故"池形拼写不改变产物"在两条 javac 腿上都是逐字节事实。`Same.class` 的
`Signature` 为 `LD$Base<Ljava/lang/String;>;`、`InnerClasses` 行 `Base=class D$Base of class D`
（`q2-replay.out` 尾段），与 root 的记录一致。

## 一条实测到的 javac 交互（记录，非本片缺陷）

同一次 javac 运行里，**同一个 flat（含 `$`）名只能被一个文件引用**：先引用的文件解析成功，后引用的
文件报 `cannot find symbol`（raw 与参数化拼写都如此；实测 `raw-first` / `par-first` 两种顺序都失败）。
本片 fixture 与测试因此**每个 javac 运行只让一个文件引用同一个 flat 名**（见
`results/03-fixtures/build-fixtures.sh` 的逐文件编译注释与 `tests/parameterized_interface_headers.rs`
的 runner 结构）。jarde 的既有恢复族（br-family）本来就是"每个 flat 名一个引用"，故本片不改变
该交互的暴露面；这条记录供后续片参考。

## 平台接口事实的转录（`javap-comparable.txt`）

本机 `rt.jar`（sha256 `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`，与
handoff 记载同源）中 `java.lang.Comparable` 的声明为
`public interface java.lang.Comparable<T extends java.lang.Object>`、类 `Signature`
`<T:Ljava/lang/Object;>Ljava/lang/Object;`（**一个**类型参数），成员 `int compareTo(T)`。
这是 `platform_header_interface_fact` 的唯一来源（见 `results/01-forensics-anchors.md` 更正 2）。
