## 1. 取证与基线（root 已完成大半）

- [x] 1.0 损坏样例、判别（静态 only/菱形显式同坏/实例字段不受影响）、jadx 有解、关联拒绝 `field_generic_body_unproved`——全部实测并归档。（root 已完成）
- [x] 1.1 **定位内联拼接点**：读 `src/class_source.rs` 的静态字段初始化呈现分支（`field_generic_body_unproved` 拒绝后的回退），找到产出 `new MN$Hold` + `ava.lang.Object)` 的拼接代码行，说明丢 `(` 与类型实参的机制；转录存证据目录。
  - **实测落点不在该拒绝分支**：损坏文本由 `src/facade.rs` 的静态成员折叠重拼循环 `project_static_fold_owner_texts` 的字段分支产出（父提交 `dccd21c3` 的 `src/facade.rs:21994` `declaration.replace_range(start..end, spelling)`——在已就地改写的字符串上继续用扫描阶段的原始偏移，从第二个名字起晚 `prefix_delta` 字节落点，吃掉名字后的文本）。逐字节模拟、门控实验与"拒绝分支改引注不能修好任何东西"的论证见 [results/01-mechanism.md](results/01-mechanism.md)；该差异待 root 更正 spec 落点（保留原文 + 追加更正段）。
- [x] 1.2 重验基线：主线二进制渲染 `MN`（两条腿）确认损坏文本仍现、`javac` exit 1；`RG`（嵌套形）同验。
  - 父提交二进制（SHA-256 `dbbcd3e9…`）实测：`MN`/`RG`/`SG` 两腿 `Holdava`=2、`javac --release 8` exit 1（`需要'('或'['`，与巡查 `results/javac-errors.txt` 同形）；`P1` 的损坏形 `new P1$Box;` 不含 `Holdava` 但同样 exit 1。见 [results/01-mechanism.md](results/01-mechanism.md)。
- [x] 1.3 冻结 fixture：`MN`（javac23 `--release 8` 腿）+ `MN8`（真 javac 8 腿，Corretto 1.8.0_432）入 `tests/fixtures/`，README 记编译命令与 SHA。
  - 目录 `tests/fixtures/recover-static-generic-field-init-text/`（`v8/` 与 `v8-javac8/` 两腿 + 源 + README）：巡查的 `MN`/`RG` 逐字复制（`v8/MN.class` 等四个文件与巡查 fixture `cmp` 相同、SHA 与巡查 `sha256.txt` 相同），另加本片两个对照 `SG`（可整类编译的同形锚）与 `P1`（同机制的非泛型对照）；编译命令与全部 SHA-256 见该目录 README。

## 2. 修复

- [x] 2.1 按定位结果修复：内联路径产出**路径 A 裸类型正确形**（`new Hold((java.lang.Object) "a")` 同构 f3），或该分支改为响亮引注；**禁止任何损坏中间态**。
  - 选路径 A：重拼循环把每次替换的落点按已累计的 `prefix_delta` 平移（`local_start`/`local_end`），产出 `static Hold f1 = new Hold((java.lang.Object) "a");`（与 f3 的 `new Hold((java.lang.Object) java.lang.Integer.valueOf(5))` 同形）；`MN`/`RG`/`SG`/`P1` 两腿渲染 diff 只有这些字段声明行（[results/03-anchors.md](results/03-anchors.md)）。
- [x] 2.2 若选路径 B（复用 f3 构造器赋值通道），先验证静态字段该通道存在且语义等价（`<clinit>` 呈现），报告说明取舍。
  - 未选路径 B，取舍已核：静态字段的初始化呈现通道就是**被证明的静态初始化组**（`project_static_initializer_group` 把 `<clinit>` 写片段内联进声明、并从装配文本里省略 `<clinit>`），其产出的声明文本本身正确（父提交报告 `fields.0.declaration = "static MN$Hold f1 = new MN$Hold((java.lang.Object) \"a\")"`）；要走 B 必须**拒绝该证明**才能让 `static { … }` 块留下，那会撤销 `class_static_initializer_projection`（EM-06）已交付的能力，且对本损坏无必要——损坏发生在该文本之后的折叠重拼里。见 [results/01-mechanism.md](results/01-mechanism.md) 1.1.1/1.1.5。
- [x] 2.3 不改 `field_generic_body_unproved` 拒绝注释本身（投影域后续片）。
  - `src/class_source.rs` 未改动（`git diff --stat` 只有 `src/facade.rs` 与 census 的 `crates/jarde-reader/src/classfile.rs`）；测试逐字钉住 `f1`/`f2`/`f3`/`field`/`nested`/`names` 的拒绝注释仍现。

## 3. 验证与验收

- [x] 3.1 主锚：`MN`/`MN8` 双腿渲染无损坏文本（`grep -c 'Holdava'` 输出 0）、`javac --release 8` exit 0、`main` 输出与原 class 一致（`a b 5`）。
  - 无损坏文本：`MN`/`RG`/`SG`/`P1` 两腿 `Holdava` 2 → 0 ✓，且渲染中不存在任何非注释行的池形名（通用判据，`P1` 的 `new P1$Box;` 不含 `Holdava` 但同样被该判据覆盖）。
  - **偏差（如实记录，待 root 裁定）**：`javac --release 8` exit 0 在 `MN`/`RG` 上**不可达**——解析错误修好后，其渲染单元只剩嵌套成员类 `Hold<T>` 自身的擦除配对错误（字段 `Signature` 投影为 `T v;` 而构造器 `Signature` 被拒、写成 `Hold(java.lang.Object arg1)` ⇒ `this.v = arg1` 为 `Object` 入 `T`），属投影域既有未决项、本片 Non-Goal（"本片不修投影本身，只修文本"），在 `--policy single-class` 呈现里同样存在。巡查 `javac-errors.txt` 只记 2 条解析错误是因为 javac 解析失败后不再进入属性检查阶段。
  - 该句在**本片作用域内**由两个对照锚完整交付：`SG`（与 `MN` 同形：静态泛型字段 + `field_generic_body_unproved` + 内联初始化；嵌套类成员无 `Signature`）与 `P1` 两腿两编译器 `javac` exit 0、运行输出与原件逐字相同（`ab5`/`1`）；`MN`/`RG` 的行为一致性由"仅手工修正那一处越界配对"的探针测得（`ab5`、`y7insf5`/`2`）。任务书写的 `a b 5` 与实测不符：`MN.main` 打印字符串拼接 `ab5`（两条腿一致），本片以"与原件逐字相同"为准。全部转录见 [results/03-anchors.md](results/03-anchors.md) 3.1/3.1.1。
- [x] 3.2 零回退：`MN.f3` 逐字节不变；corpus 双腿扫描差异类仅为静态泛型字段初始化形（如实记录数量）；`RG` 宿主的非静态字段不受影响。
  - `MN.f3` 与其构造器赋值行两腿逐字节不变（不在 diff 中）；`RG` 的静态字段 `names`（初始化不点名被折叠类）与全部方法文本（`pick`/`consume`/`newInner`/`main`）逐字节不变；`RG` 无实例字段，实例字段的零回退对照由 `MN.f3` 承担。
  - corpus 三档双腿差分（单类姿态全语料 / 折叠姿态每个 loose 根类 / 每个 jar 条目；自检先行 + pass A moved 必须为 0）：数字与逐类分类见 [results/02-corpus-sweep.out](results/02-corpus-sweep.out) 与 [results/02-corpus-delta.md](results/02-corpus-delta.md)。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy（ci.yml 46-76 逐字）+ openspec strict + `git diff --check` + 再生 fingerprint。
  - 命令与尾部逐字记录见 [results/04-gates.md](results/04-gates.md)。
- [ ] 3.4 root 独立复核：拼接 bug 机制转录、修复形态合规（无损坏中间态）、主锚/零回退实测；关闭 summary.md 呈现缺陷登记行。（留 root）
