# `recover-unicode-identifiers` 冻结输入

变更 `openspec/changes/recover-unicode-identifiers` 的输入。全部 `.class` 由 **真 javac 8**（Amazon
Corretto 1.8.0_432，`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home`）
编译；第二条 javac 腿（工具链 `javac --release 8 -encoding UTF-8`）在测试内由同一份冻结源码现场编译，
不在此目录冻结产物。

## 生成命令（逐字）

```text
JAVA8=/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home
"$JAVA8/bin/javac" -encoding UTF-8 -d ut ut/UT.java
"$JAVA8/bin/javac" -encoding UTF-8 -g -d loc loc/Loc.java
```

`ut/UT.java` 是 [unicode-identifier 巡查](../../../openspec/evidence/java-syntax-2026-10-05/unicode-identifier-patrol/README.md)
的固定义本（字段 `变量`/`描述`、方法 `方法`、嵌套类 `内部类`、其字段 `名字`），逐字复制；
`-g` 只加在 `loc` 上——`Loc` 要的就是 `LocalVariableTable` 里的中文**局部名**（`数量`），
锚类 `UT` 用 javac 的默认调试信息（行号 + 源文件，无局部表），与巡查一致。

## 逐位身份（sha256）

| 文件 | 字节 | sha256 |
| --- | --- | --- |
| `ut/UT.java` | 435 | `2b122dce339956497a1e3ccef1b6152977cb6f44bf34fb59afa0076f13010d85` |
| `ut/UT.class` | 955 | `00557f2a5007ad47a9d60839339c94de481e3cb5ee47da6e6255c92823a49d5c` |
| `ut/UT$内部类.class` | 305 | `cda8bae084dac3852d7ce1ede1cde654041e452f4567a7047809f876a0b13de3` |
| `loc/Loc.java` | 204 | `0dd68c411f75df0534c29965a36fbcf67924da393a52728eaf2465a5477bdd50` |
| `loc/Loc.class` | 600 | `39d5a01c4acdd1ae16e85101af753773eb56a8ab3efe37cd5fd5698bb9ddac08` |
| `loc/Loc.punct.class` | 600 | `ac2e4a9248b0c4cee40f2cfc3baac9b6d0c30518e992afc32778cf36f915f6bf` |

巡查 `ut.jar` 的 `UT.class` 与本目录的不同字节（`740d710f…` vs `00557f2a…`，同为 52.0）：巡查那份
未记编译命令，本片以真 javac 8 重编并冻结自己的产物，故以本目录的 sha256 为准，两条腿都在测试内重放。

## `Loc.punct.class` 的等价长替换（可复现）

`Loc.punct.class` 是 `Loc.class` 的**逐字节等价长替换**：`LocalVariableTable` 里局部名
`数量`（UTF-8 `E6 95 B0 E9 87 8F`，6 字节）换成 `名。`（`E5 90 8D E3 80 82`，6 字节，`。` 是表外字符）。
文件中该 6 字节序列恰出现一次（该名只被局部表引用），替换后 class 结构自洽、可加载：

```python
src = open('loc/Loc.class','rb').read()
old, new = '数量'.encode('utf-8'), '名。'.encode('utf-8')
assert len(old) == len(new) == 6 and src.count(old) == 1
open('loc/Loc.punct.class','wb').write(src.replace(old, new))
```

替换后该名字仍是 `Java 字母 + 表外标点`：声明与使用都必须走别名（`__`），这正是「放宽的是 JLS 的字母与
数字，不是『非 ASCII 一律放行』」的端到端对照。

## `ascii/`：修前呈现的逐字节控制

`ascii/*.jarde.java` 是**修前二进制**（`git stash` 掉本变更后、`cargo build -p jarde-cli` 出来的 CLI，
对应基线提交 `816ee7b6`）对既有 patrol fixture 的渲染，测试
`tests/recover_unicode_identifiers.rs::the_ascii_presentations_are_byte_for_byte_the_pre_change_ones`
断言修后渲染与之**逐字节相等**：

| 冻结文本 | 输入（class 字节在 evidence 内，测试内 `include_bytes!`） | 渲染形状 |
| --- | --- | --- |
| `F1.jarde.java`（2321 B，`52553f18…`） | `cf10-crossing-array-read-patrol/fixture/F1.class` | 单条目 jar，`--class F1` |
| `SB.jarde.java`（5234 B，`e9096450…`） | `charsequence-arg-widening-patrol/fixture/SB.class` | 单条目 jar，`--class SB` |
| `RG.jarde.java`（3247 B，`dbb41f17…`） | `recursive-generic-patrol/fixture/RG.class` | 单条目 jar，`--class RG` |
| `package-info.jarde.java`（542 B，`152bb516…`） | `package-info-patrol/fixture/pi.jar`（原样） | jar，`--class com/example/package-info` |

三条 `.class` 以**单条目 jar** 形状渲染（`class-source` 对独立 class 文件与 jar 的 `plain_jar` 环境
策略不同，测试用 jar 形状，冻结文本也以同一形状取），`pi.jar` 直接使用巡查的 jar 字节。
`package-info` 那份同时钉住本变更**未触及**的边界：类声明行 `interface package-info {` 的名字仍按池
原样拼接（声明行不经 `is_java_identifier`；本变更只改判定本身，不新增声明行的合法性门）。

## 读取者

`tests/recover_unicode_identifiers.rs`（锚、双腿往返、别名对照、ASCII 逐字节控制）；
`crates/jarde-reader/src/classfile.rs` 的 fixture 人口普查把本目录 4 个 `.class` 计入
（+4 类、+11 Code 体），`tests/p5_corpus_fingerprint.rs` 固定本目录全部输入字节。
