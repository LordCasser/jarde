## 1. 固定证据与控制基线

- [x] 1.1 接受实际同IR诊断、核对周边声明/reuse/调用架构并冻结最小设计。验收：Phi→Load→Return真实SSA闭合、首写/后写同identity、准确descriptor与原资源失败单列；设计只准直接Object Return，无新机制；前片自身CI/最终clean真实交付。
- [x] 1.2 冻结literal正向debug/no-debug及合法read/接口/数组/参数/slot/handler控制，root采集旧产品完整类基线。验收：原/JADX/Jarde双JDK的compile、runtime与物理BCI分别记录；slot reuse按实际class/reuse事实，不从源码推定；拒绝的完整类不得记成运行通过。实际debug93/none97命令与两个独立checker exit0仅接受观察；两profile完整Jarde类集合各四次compile1/no runtime，physical slot2/LVT range已核，不声称SSA身份已采集。

## 2. 最小类型决定与永久回归

- [x] 2.1 Luna私有候选经root对抗审查后应用。验收：只普通body first-write窄scalar/reference、准确Object stored producer触发，all writes/paths与all Load唯一直接Object Return闭合后统一Object；参数/header/专门type/数组/handler门保持，缺失读证明准确conflict；所有新增扫描计费/poll/Stop，无identity/LUB/pass或cast。实际root v4 core4/0/0、公开API3/0/0；真实slot2旧/新LVT映射同index0，旧receiver读拒绝，未从debug范围猜split。
- [x] 2.2 添加必要真实永久API/physical来源及反例，实测late Stop与pre-cancel。验收：正向debug/no-debug宣告Object并保留seed/next/return，default/all/no-rule-details同正文/map，handler/slot/read边界准确归因，不发布部分文本/来源或错误Object化。实际公开API4/0/0；exact单方法完整IR后debug331→330/none347→346末尾Java计费中止、无产物，核心独立finalReturnconsumer Stop/precancel通过；BCI2不用于推断执行早晚，written字段实际是output_bytes。

## 3. 完整对照与主线交付

- [x] 3.1 root新CLI完整类双JDK对照并独立验收。验收：未手修生成类编译运行的值、状态、异常/效果顺序同原；JADX差异与Jarde仍拒绝边界单列；准确CLI字节及来源绑定。实际debug97/none101命令、专门checker root-v2 exit0、每profile四条目标整类29行编译/运行同原；其它全生成combined失败与12 refusal map gaps/profile单列，不扩大行为接受范围。
- [x] 3.2 fresh全工作区、fmt/当前CI Clippy/global strict、冻结最终CLI。验收：真实targets/named outcomes与源码闭包完整核验、5GiB free/target1GiB一秒守卫/default stack/finally clean；失败不得拼接为成功。
- [x] 3.3 提交推送确切产品、独立验收其自身CI。验收：确切head/run四jobs/全部steps和双seed具名结果与本地/CLI绑定，不借前片或文档CI。
- [x] 3.4 更新账本/verification/handoff并交付干净main。验收：文档推送后按真实HEAD核main/origin/remote仅main、aux detached干净祖先、全部无target；关闭后另作外置最终audit，不预写未知SHA，CF11/71/612不增。
