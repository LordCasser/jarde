# 任务 1.1/1.2 证据：损坏文本的拼接点定位、机制、基线重验（coder，2026-10-06）

实现者：coder subagent（隔离 worktree）。基线 = 本 change 的父提交 `dccd21c3`
（`git rev-parse HEAD` 实测，工作区干净）。构建：`cargo build --release --locked -p jarde-cli`，
基线二进制快照 `/tmp/rsgfi/bin/jarde-cli-base`（SHA-256
`dbbcd3e996f6da23f0a636fa72372672d35f1ffc9ab79a9666830dfcafd06bea`）。

## 结论摘要（先读这段）

1. **损坏文本的拼接点不在 `class_source.rs` 的 `field_generic_body_unproved` 回退分支**（任务书的
   pinned premise 落点假设），而在 `src/facade.rs` 的**静态成员折叠的名字重拼循环**
   `project_static_fold_owner_texts` 的字段分支：父提交 `src/facade.rs:21994`
   `declaration.replace_range(start..end, spelling);`——它**在已经就地改写过的字符串上继续使用扫描阶段
   的原始字节偏移**。
2. 该字段声明文本**本身是正确的**（含 `<clinit>` 证明内联进来的初始化片段），损坏发生在它被重拼时：
   从第二个名字出现处起，每次替换都晚 `prefix_delta` 字节落点，吃掉名字后面紧跟的文本。
   `MN$Hold`（7 字节）重拼为 `Hold`（4 字节）⇒ `prefix_delta = -3`，于是 `((java.lang.Object)` 的前 3 字节
   `((j` 被吃掉 —— 逐字复现巡查记录 `new MN$Holdava.lang.Object) "a")`。
3. **触发条件不是泛型**：判别变量是"字段声明里同一个被折叠类名出现 ≥2 次"，而这在静态字段初始化被
   `<clinit>` 证明内联进声明后必然成立（字段自身的池形类型 + 初始化片段里的构造调用）。控制锚 `P1`
   （**非泛型**嵌套类，全类无 `Signature`）在父提交同样损坏：`static Box b = new P1$Box;`（整个实参表被吃）。
   `field_generic_body_unproved` 拒绝只是"字段类型保持池形"的一个来源，不是损坏的原因——把该分支改为
   响亮引注**无法**修好任何东西（声明文本仍会带着内联初始化片段进入同一个重拼循环）。
4. 修复 = 重拼循环把每次替换的落点按已累计的 `prefix_delta` 平移（与同一函数里三个兄弟循环一致；那三个
   循环用 `.rev()` 倒序改写，所以本来就没有这个问题）。产出即路径 A 裸类型正确形，与 f3 既有退化形同构。

## 1.1 机制（读码 + 逐字节模拟 + 门控实验）

### 1.1.1 上游：初始化内联拼接（这条是对的）

`src/facade.rs` 的 `project_static_initializer_group`（父提交 `src/facade.rs:10180-10188`）：

```rust
    // The plan is complete and every output byte was paid for; only now publish any initializer.
    for (physical_index, fragment) in staged_initializers {
        fields[physical_index]
            .declaration
            .as_mut()
            .expect("the staged declaration was checked")
            .push_str(&fragment);
    }
```

它把 `<clinit>` 里被证明的写片段（` = new MN$Hold((java.lang.Object) "a")`）追加到字段声明上。父提交下
**报告**里的字段声明就是这个正确文本（`results/03-anchors.md` 的 `fields.N.declaration` 转录）：

```
fields.0.declaration = "static MN$Hold f1 = new MN$Hold((java.lang.Object) \"a\")"
fields.1.declaration = "static MN$Hold f2 = new MN$Hold((java.lang.Object) \"b\")"
fields.2.declaration = "static MN$Hold f3"
```

`field_generic_body_unproved`（`src/class_source.rs:5989` 常量、5986-5992 分支）决定的是**该字段类型保持
物理描述符拼写**（`MN$Hold` 而非 `Hold<String>`），因此声明里池形名出现两次而不是一次。它不是损坏点。

### 1.1.2 损坏点：折叠重拼循环的前向就地改写（父提交 `src/facade.rs:21931-22015`）

```rust
        let mut derived = Vec::new();
        let mut prefix_delta = 0isize;                                   // 21987
        for (start, end, spelling, ...) in &edits {                      // 21988
            let final_start = start
                .checked_add_signed(prefix_delta)                        // 已累计的 delta 只用于 derived 记录
                .and_then(|offset| offset.checked_add(staged_prefix))
                .ok_or_else(|| Error::invalid_input("static_fold_offset", "span overflow"))?;
            let final_end = final_start + spelling.len();
            declaration.replace_range(start..end, spelling);             // 21994 ← 用的是扫描阶段的原始偏移
            ...
            prefix_delta += spelling.len() as isize - (end - start) as isize;   // 22011
        }
```

`declaration` 是就地改写的 `String`（`declaration.replace_range` 会改变长度），所以第 i 次替换时，
原始偏移 `start..end` 已经不等于该名字当前的位置：它应平移 `prefix_delta`。第一次替换之后字符串长度
变化了 `delta`，第二次替换就晚了 `delta` 字节落点，把名字**后面**的 `delta` 个字节连同名字一起替换掉。

同一函数内的三个兄弟循环不受影响：方法声明的重拼（`22222-22223`）、方法体的 pending edits（`22517`）、
嵌套枚举的声明重拼（`23348-23349`）都按 `.rev()` **倒序**改写，倒序时后续编辑的偏移不受前面编辑影响。

### 1.1.3 逐字节模拟（复现巡查的逐字记录）

对父提交下 `MN`/`RG` 的**报告声明文本**（上引 `fields.N.declaration`）按上述循环做纯文本模拟，
`spelling = "Hold"`，`pool_dotted = "MN$Hold"` / `"RG$Hold"`：

```text
MN  decl = static MN$Hold f1 = new MN$Hold((java.lang.Object) "a")
    spans  = [(7, 14), (24, 31)]
    edit 1 = replace(7..14,  "Hold")  -> static Hold f1 = new MN$Hold((java.lang.Object) "a")   delta -3
    edit 2 = replace(24..31, "Hold")  -> static Hold f1 = new MN$Holdava.lang.Object) "a")
    observed (父提交二进制渲染)        = static Hold f1 = new MN$Holdava.lang.Object) "a");      ← 逐字相同

RG  decl = static RG$Hold nested = new RG$Hold((java.lang.Object) new RG$Hold((java.lang.Object) java.lang.Integer.valueOf(5)))
    spans  = [(7, 14), (28, 35), (59, 66)]
    模拟结果 = static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)))
    observed = static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));   ← 逐字相同
```

两次模拟与两个锚的父提交渲染**逐字节相同**（含巡查 `findings.txt` 记录的嵌套形片段
`new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) …`），机制据此判定成立。

### 1.1.4 门控实验（只改这一处，看锚的行为是否变化）

门控实验 = 本 change 的修复本身（`declaration.replace_range(local_start..local_end, spelling)`，
`local_start = start + prefix_delta`），编译后重渲染四个锚的两条腿：

| 锚 | 父提交（`dccd21c3` 二进制） | 修复后 |
| --- | --- | --- |
| `MN.f1` | `static Hold f1 = new MN$Holdava.lang.Object) "a");` | `static Hold f1 = new Hold((java.lang.Object) "a");` |
| `MN.f2` | `static Hold f2 = new MN$Holdava.lang.Object) "b");` | `static Hold f2 = new Hold((java.lang.Object) "b");` |
| `RG.field` | `static Hold field = new RG$Holdava.lang.Object) "sf");` | `static Hold field = new Hold((java.lang.Object) "sf");` |
| `RG.nested` | `static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));` | `static Hold nested = new Hold((java.lang.Object) new Hold((java.lang.Object) java.lang.Integer.valueOf(5)));` |
| `SG.f1` | `static Hold f1 = new SG$Holdava.lang.Object) "a");` | `static Hold f1 = new Hold((java.lang.Object) "a");` |
| `P1.b` | `static Box b = new P1$Box;` | `static Box b = new Box(1);` |

两条腿（javac23 `--release 8` 与真 javac 8 的产物）逐字节给出同一结果。四处改动**只**落在这些字段声明行
（`diff` 全文见 `results/03-anchors.md`），其余行（含 `f3` 的实例字段路径、全部方法体、全部拒绝注释）逐字节不变。

### 1.1.5 与 pinned premise 的差异（供 root 裁定）

| 任务书 pinned premise | 实测 |
| --- | --- |
| 落点 = `src/class_source.rs` 的 `field_generic_body_unproved` 拒绝后的回退分支 | 落点 = `src/facade.rs` 的静态成员折叠字段重拼循环（父提交 21994） |
| 触发链 = 静态字段 + 泛型 Signature 拒绝 + 初始化含泛型类构造调用 | 判别变量 = 字段声明里被折叠类名出现 ≥2 次（由"静态初始化被证明内联"造成）；泛型与拒绝都不是必要条件（`P1` 非泛型同坏） |
| "该分支改为响亮引注"是可选修法之一 | 在拒绝分支做任何事都**不能**修好损坏文本：损坏发生在声明文本被构建**之后**的折叠重拼里 |

设计文档 `design.md` 的 Open Question 1 已把"内联拼接点的具体位置"留给实现者定位并转录
（"root 未定位到具体拼接行"），`design.md` 决策 2 要求"修复 = 拼接点产出的文本必须与路径 A 一致"，
因此本实现按实测落点施工：修复后**拼接点产出的文本即路径 A 裸类型正确形**，与 f3 既有退化形同构。
premise 中的文件/分支名差异请 root 在验收时更正（保留原文 + 追加更正段）。

## 1.2 基线重验（父提交二进制，两条腿，fold 姿态）

harness：`/tmp/rsgfi/harness.sh`（渲染 → 自述头断言 → `Holdava` 计数 → 把渲染文本写成
`<CLASS>.java` → `javac --release 8` 与真 javac 8 两腿编译 → `-Xverify:all` 运行）。自检：渲染首行必须带
`// jarde: presentation of`，否则脚本以 `NO-SELF-HEADER` 退出（脚本第一版因文件名拼接 bug 被该断言拦下，
见 `results/03-anchors.md` 的 harness 自检记录）。

```text
class=MN label=base broken=2 rel8_exit=1 javac8_exit=1 original='ab5' rel8_run='-' javac8_run='-'
class=RG label=base broken=2 rel8_exit=1 javac8_exit=1 original='y7insf5\n2' rel8_run='-' javac8_run='-'
class=SG label=base broken=2 rel8_exit=1 javac8_exit=1 original='ab5' rel8_run='-' javac8_run='-'
class=P1 label=base broken=0 rel8_exit=1 javac8_exit=1 original='1' rel8_run='-' javac8_run='-'
（两条腿各一份，逐字节相同的渲染与相同的退出码）
```

- `broken` = `grep -c 'Holdava'`；`P1` 的损坏文本不含 `Holdava`（它是 `static Box b = new P1$Box;`），
  故计数为 0 但 `javac` 仍 exit 1——这也是"`Holdava` 计数不能当作通用损坏探测器"的实证，
  3.2 的普查因此同时使用"非注释行含池形名"这一通用判据。
- 父提交 `javac --release 8` 的错误（`MN`，v8 腿）逐字与巡查 `results/javac-errors.txt` 同形：

```text
MN.java:5: 错误: 需要'('或'['
    static Hold f1 = new MN$Holdava.lang.Object) "a");
                                               ^
MN.java:8: 错误: 需要'('或'['
    static Hold f2 = new MN$Holdava.lang.Object) "b");
```

- `original` 列是 fixture 自己的 class 在本机 JVM 上的实测输出（先跑原件再谈渲染，避免把期望值写成假设）。
  任务书写的 `a b 5` 与实测 `ab5` 不同：`MN.main` 打印 `f1.v + f2.v + f3.v` 的字符串拼接结果，
  实测为 `ab5`（两个 fixture 腿一致），本 change 以"与原件输出逐字相同"为准，记录实测值。
- 单类姿态（`--policy single-class`）在父提交**不损坏**（池形 `static MN$Hold f1 = new MN$Hold((java.lang.Object) "a");`，
  与巡查归档的 `jarde-RG-after-platform-implementer-tables.txt` 同形）——损坏只在家族折叠姿态出现，
  这是 3.2 普查分三档（单类/折叠/jar）的原因。
