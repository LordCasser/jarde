# 构造条件值恢复验收（在制）

当前 6/9：当前 production/永久 fixtures/API/Stop 与 strict CLI 完整固定/正例 runtime 已实际接受。全workspace验证正在运行；lint/自身CI/main交付未接受。

1.1 固定历史基线与当前同 IR 首拒已 root 接受，见 results/fixed-initial-observations-root-v1.json。29 命令完整对照中的旧 CLI 是 v10，不是上一产品最终 CLI；13 blocks/18 edges 的真实诊断确认 constructor96→Phi99→Return99，仅说明 Site 首拒，不能推断 Builder 已通过。

1.2 新控制基线已 root 接受，见 results/control-baseline-root-v1/root-acceptance.json。39 命令/78 raw，双 JDK 原类与 JADX 四组完整类六 runtime 腿，19 行同 oracle（SHA256 a06d6cc59147b779796efa914406f8ff35028263a534906893de5ee2dabf5c7e）。包含 identity、null 构造/实参/条件异常、字段不写入失败值和 handler。Jarde 12 compile 失败/no runtime；生成源码不改，隔离采用同一生成源码。CLI final-v3 与实际 4718 源闭包/上一 build 绑定。

Collector actual exit0；v5 checker actual exit1 是它限制 exception_count==0，本轮加入真实 catch 后暴露。v6 最小扩展完整 physical Code exception table，逐项对 javap 有序表和 instruction endpoints 核验，actual exit0。旧 failure 原样保存，不将其覆写成通过。验证结果 quality gaps 仍在，root 只接受观察与 original/JADX oracle。BLAKE3 不独立重算，只核 byte length/digest shape、physical location 与 default/all identity；SHA256 独立核实 bytes。

Luna v1 静态候选未应用；v2 消除普通 constructor 二次 writes/uses 扫描/计费及额外 refusal Vec 排序。root 逐项基线 pin 验证后应用三个文件，并修正 iterator 共用显式 lifetime 与 move capture。候选 strict one-hop 选择不拥有 CFG；完整条件 branch/effect 仍必须由现有 Builder 证明；最终 AST 一次扫描 sparse exact method-local constructor New count==1，summary/refusal/selected record 同一判定。详 results/private-candidate-v2/root-review.md。真实 shared/nested/Stop/API 与完整固定/控制新 CLI 流水线仍待验收，不借静态证明和上一产品测试。

资源采用用户明确批准的 5GiB free/target1GiB、1 秒进程组中止、默认 Rust stack、完成后 cargo clean。root 独占 Cargo/JDK/JADX/CLI/shared/Git，Luna 仅 private preparation；不新增 branch/worktree。

第一轮候选 v2 真实结果见 results/first-candidate-v2-observations/README.md：固定完整class 双JDK/default-all四 Jarde runtime同原，所有方法物理BCI/map完整；控制Positive四组同源隔离完整class runtime同19行，Refusals/combined八 compile失败，不声称两个生成类combined成功。v2仅观察验收，当前source后补Direct provenance和否例须新构建/对照。

新增三个canonical Java8 class/source的fixture字节固定，reader census静态测量增量3/21/1/34/0，候选总1103/4911/469/2987/10，Rust sweep 1/0/0 已实际通过。公开lateStop取指定target body的实测累计AnalysisSteps-1（其后clinit不是目标），严格要求同PhysicalMethodId的body stopped、真实位置、无部分text/map/content/news。v4/v5 真实失败见 results/focused-api-failures-root-v4-v5/README.md。v4 编译诊断字段修正后 v5 指定方法 lateStop/pre-cancel 和四正例均通过；两个 class-level range 假设、X4 Region/Builder fallback 假设修订待实跑。fingerprint/新 CLI 尚未执行，失败不提前勾任务。

当前v6严格门/公开API验收见 results/focused-acceptance-root-v6：5公开tests、updater1/check5-0-1、新CLI/clean均实际通过，source closure4727，原manifest仅增加7本轮files；相同production/fixture绑定v4库360/census1成功。2.1–2.3已据此接受。新CLI完整fixed/controls双JDK三方正在运行，3.1及完整workspace/lint/自身CI/main交付仍未接受。

3.1当前strict CLI fresh72命令/144raw经v7/root独立actualexit0接受，见 results/current-cli-runtime-acceptance-root-v6。固定四Jarde wholeclass与Positive四same-source isolated wholeclass分别同双JDK oracle；控制combined/Refusals八compile失败/no runtime、三个成员physical缺口保留。整个CF11/71计数不增。

Fresh workspace v1及独立acceptor actualexit0：95批/361targets/98cmd/196raw、3473/0/97/3570named，12新增关键names恰好一次ok，source4727恒同/默认stack/guard/clean。详results/full-workspace-root-v1。CI范围lint/strict/finalCLI尚在运行，3.2整体未勾。
