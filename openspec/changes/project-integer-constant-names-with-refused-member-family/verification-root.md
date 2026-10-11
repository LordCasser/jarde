# Root 验收记录

当前 11/13；本地实现与完整对照已接受，自身产品 CI 和最终主线交付待完成。持续目标及整 CF12 保持开放。

生产只在 `src/facade.rs` 既有调用 gate 增加 `Refused { child: None }` alternative。Complete、三个 sibling Absent 条件及整数名称投影函数字节不变；childSome、RefusedPair、所有 Prepared variants 仍排除。没有新增机制、依赖、入口或 reason 分支。唯一新增核心测试读真实冻结 root/独立 Inner，核对字段、物理方法、派生 UTF8 spans 和 BCI。真实 family fixtures 验证各状态自身，未冒充每一种状态同时存在整数名称候选。

聚焦 12 命令通过，fmt、CI 同范围 Clippy、库 221/0、唯一核心 1/0、五组 integration 20/0、102/0、7/0、4/0、8/0。fresh 完整 workspace 95 批覆盖 metadata 全部 359 targets：3445 passed/0 failed/97 ignored；8 个合法零测试目标也按实际 artifact/header/result 逐一绑定。动态 4695 源码、literal include 与 fixture 输入前后恒同，独立 verifier 实际 exit0。完整输入 CLI build+clean 两命令通过，冻结 CLI SHA `8e70234797af43e114e94cd08ba0a8f120d59d0e9c638a20a8cf64e9876fd64b`、60820032 bytes、0555；与聚焦 CLI 逐字相同。target 已清理。strict 实际 349/0。

冻结 upstream Java SHA `8fad0c3325b33685dbb985bc9e9f55c130c8c678e4e333b60cd66dd02875ed77`，与本地 JADX checkout 相同。源代码 root 使用 CONST_ABC/CONST_CDE，Inner 使用私有 CONST_CDE_PRIVATE 并引用外层 CONST_ABC。固定 JADX 完整输出 root 恢复这两个本类名称；Inner case 保留3294，return 使用外类限定名。新 Jarde 独立 root 恢复两个本类名称，独立 Inner 恢复私有 case 名称而 return2748 保持数字。JADX local/global 同值冲突的保守选择不应阻止已证明的本类私有字段投影；单 class 输入缺 outer 字节，跨类绑定另立债务。

实际完整 replay v5 共32命令：冻结旧 Bafb CLI 与新 CLI 各两个 class×default/all，随后 Java8/23×default/all 四组，每组完整 JADX/Jarde/probe 编译和原/JADX/Jarde `-Xverify:all` 运行。全部 exit0，每腿10输入行、总120行观察一致；完整 class 集均为两个物理类。Jarde 完整源码八份逐字等于新 CLI text，没有改写后编译。两个独立只读 verifier 经 root 实际重跑通过；物理 recovery text/map/identity/BCI 恒同，default/all 语义相同。root 的派生名锚在 f1 BCI1/20，Inner 在自身 f1 BCI1。精确计费变化为两个 class 都 AnalysisSteps+2、IrItems+15；OutputBytes root+1429、Inner+1423，其余非时间计数不变。

v3 在 CLI 前因旧记录额外 mode 字段失败；v4 八次 render 后因 JDK manifest legs 为列表失败。这些真实失败与原 raw 保留，未计作完整 replay 通过。证据分别在 `results/focused-validation-root-v1/`、`results/full-workspace-final-cli-root-v1/`、`results/complete-labels-replay-root-v1/`，原 absolute private 路径不改写，归档 inventory 绑定按字节复制的文件。

所有 Rust 构建遵守用户批准的 5GiB free/target1GiB、一秒进程组守卫；完整 workspace 观察峰值444242729 bytes，最终CLI峰值360190581 bytes，完成后 target 不存在。下一步为准确产品 commit/push、自身 CI 全部 jobs/steps/双 seed 完整目标日志验收及主线清理。
