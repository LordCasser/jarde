## 1. 同次事实与对照基线

- [x] 1.1 root接受三profile同次结构观察，核周边AST投影架构并冻结平台契约。验收：真实45Code/operations/SSA/effects逐BCI唯一配对、参数读写/物理Stringcast与add/remove包装、三种结构相同但usage单列；原v3采集失败不改写。Java8/23 javap平台List/Collection/Iterator原始签名相符，前片最终main审计真实通过。
- [x] 1.2 冻结固定CF11与不同元素型/overload/物理cast/raw/alias/重赋值/wildcard/handler/停止控制的原/JADX/旧Jarde对照。验收：root真实编译debug与无debug，双JDK完整类和Runner，值/状态/异常/效果顺序分别保存；拒绝编译失败no-runtime不计通过，实际slot/SSA归因，不从源码猜。

## 2. 一个完整源级投影

- [x] 2.1 Luna私有候选经root对抗审查后应用：多block完整VoidBody census及全部List参数真实用途证明。验收：至少固定String与不同元素型成功，未知/漏读/重复/参数写/alias/handler失败；保留unread真实含义、计费/poll/Stop与现有layer，不新增solver/pass/identity/carrier服务。
- [x] 2.2 复用保留AST和现有精确argument-edit接口、事务overlay同时安装泛型声明与调用正文。验收：add仅删除已证明synthetic包装、physicalStringcast和remove(Object)/index重载准确保留，旧raw路径不变；wrong method/BCI/opcode/target/position/physicalcast/multiplicity均拒绝，失败不安装部分overlay，不改全局Object cast。
- [x] 2.3 永久公开API/source-map/三profile及precancel/精确lateStop回归。验收：未手修全文List<S>/泛型调用合法，source原始BCI完整与UTF8映射有效，default/all/noRuleDetails同正文；实测新增证明后期Stop无部分输出，所有新增工作有原维度计费。

## 3. 整类对照与交付

- [x] 3.1 root最终CLI双profile/双JDK原/JADX/Jarde固定及控制完整源码对照并独立验收。验收：实际新生成完整类编译运行、值/集合状态/异常/效果顺序同原；fixed8路径含null/CME/heap污染等必要边界扩展，不用手工改源码充当成功，独立拒绝和map gaps准确记录。
- [x] 3.2 fresh全workspace、fmt/当前CIClippy/globalstrict，冻结来源与最终CLI。验收：真实具名结果/目标exact-once、完整源码闭包同byte identity、5GiB/target1GiB一秒守卫/defaultstack/finallyclean，失败不拼接。
- [ ] 3.3 提交推送确切产品并独立接受自身CI。验收：准确HEAD/run所有jobs/steps与双seed结果逐目标匹配本地/来源/CLI，不借文档或旧CI。
- [ ] 3.4 更新账本/verification/handoff并fresh main交付。验收：文档推送后main=origin=remote仅main、14aux detached干净主线祖先/全部无target；关闭后实际finalHEAD另做外置audit，固定CF11/71/612完成数不擅增。
