## 1. 取证与基线

- [x] 1.1 重放固定 F1 家族 fam.jar（SHA 核对）：读 build.rs:21191 附近 interface-special 证明选择，确认失败环节与消费点；记录双形基线。（fam.jar 七成员与 `results/fixture-sha256.txt` 逐一相符；主线重放双形拒绝与巡查记录逐字一致。失败环节＝facade.rs `interface_source_type_accessible` 的 `!owner.contains(&b'$')` 首片排除——快照跨类事实读取链路完备且顶层接口正路径已有恢复，仅嵌套限定符在读事实前被拒；消费点 build.rs:21179 已有"直接超接口"白名单雏形＋按 (owner,name,descriptor) 查证明表。取证见 `sd/README.md` §1）
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：菱形+自身覆写混合、带参/void 默认方法限定调用、限定符经 extends 间接（拒绝）、目标 abstract（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。（`sd/variants/` SDDiamond〔菱形双限定+单限定混合+void `log`+带参 `greet`〕与包内 `pkg/SDPacked`；两个负例为 javac 拒编译源形，由合法邻居类经 `sd/patch-negatives.py` 单索引补丁所得：abstract 目标形 `-Xverify:all` 链接通过；**间接限定符形 HotSpot 校验器自拒**〔"interface method reference is in an indirect superinterface"〕——直接性是平台级不变量，取证如实记录为不可加载而非 verifier 有效；前后行为见 `sd/results-sd/before|after/`，负例 before/after 逐字节相同）

## 2. 快照事实证明与呈现

- [x] 2.1 interface-special 证明接入快照 header/成员事实（design 决策 1–2）；F1$Diamond/F1$Reabstract$Impl 恢复、F1 家族整 jar 重编运行一致（`AB`/`I:A`）；负例保持拒绝。（修改仅删 `$` 排除一处；`F1$Diamond→return F1$A.super.name() + F1$B.super.name();`、`Impl→return "I:" + F1$A.super.name();`；F1 七类 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行 == o1.out 逐字；F2 对照为既有 `<clinit>` 尾 `return;` 拼写偏差〔与巡查基线逐字节相同〕致其重编不闭合，属既有行为、未受本变更影响；间接/abstract 负例拒绝文案前后逐字相同）
- [x] 2.2 普通 super/this、invokestatic 接口方法、既有 invocation 通道 diff 断言逐字不变；预算/取消原子性不变。（`sd/results-sd/channel-diff/`：SpecialProbe〔普通 super/private 接收者/副作用实参/既有 `DefaultProbe.super.value()` 正路径〕与 F1/F1$A/F1$B/F1$Reabstract/F1$Reabstract$C/F2 输出主线腿 vs 修复腿逐字节相同；仅两目标方法翻正。预算/取消由 `interface_hierarchy_observes_dependency_depth_and_cancellation` 与全量测试覆盖，未改计费结构）

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 snapshot-hierarchy-widening、member-family、super/this 系既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。（实跑更宽口径 `--all-targets --all-features --locked --no-fail-fast`：exit 0、291 test-result-ok、0 failed；新 fixture 触发 p5_corpus_fingerprint 再生成〔+12 类〕与 jarde-reader fixture 计数卫兵 440→452/2118→2139 同步；fmt 干净；clippy 按 ci.yml 实有 29 项 `-A`＋`-D warnings` 干净；openspec strict 247/247〔与上一已合入切片账本口径一致〕）
- [x] 3.2 F1 家族与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（原类与 Jarde 腿全部逐字一致〔F1==o1.out、SDDiamond/SDIndirect/SDAbstract/pkg.SDPacked==原类〕；固定 JADX dev 对**全部**限定 super 默认调用一律去限定符转写〔`super.name()` 等〕、其输出不可编译〔`找不到符号 name()`〕，run 腿不可产出——按 handoff 约束记录其错误转写而不照搬；JADX 腿源码与编译错误存 `sd/results-sd/*.jadx.*`，全量 SHA 见 `sd/results-sd/sha256.txt`）
- [ ] 3.3 root 独立复核直接性判据、成员事实读取与三方行为，更新 DT/EM 账本与巡查记录。
