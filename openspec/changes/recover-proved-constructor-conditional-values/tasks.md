## 1. 真实基线与所有权证据

- [x] 1.1 接受固定SequentialLoops2双JDK完整三方基线、同次IR首拒和JADX条件值/本仓架构审阅。验收：29命令原始失败、13blocks/18edges、ctor96→Phi99→Return99唯一关系准确；诊断源码恢复/clean，与后续Builder尚未实测分开。
- [x] 1.2 冻结完整控制Java8源码与双JDK三方旧基线。验收：单臂/双臂构造、调用实参次序、条件及实参异常、对象身份、共享值/消费者、独立effect/handler边与claimed field控制均有真实class/oracle/完整raw；不编辑生成源码消除失败。

## 2. 局部所有权与最终呈现一致性

- [x] 2.1 Luna准备私有最小Site一跳候选并由root审查应用。验收：准确constructor输出、唯一operand use、live两显式输入stack Phi、唯一同join消费者一次读取、原writer/field/args全部门及计费保持；不递归alias/Phi、不新增CFG proof，排除self/local/replaced/shared/nested shapes。
- [x] 2.2 复用最终AST扫描和统一Site呈现判定。验收：只count候选direct method-local constructor New，零/一/重复/foreign origins及method reference正确；summary/gap/selected NewRecord同一判定，普通Site旧路径不混修，default/all/noRuleDetails/range一致；不新增可变Builder状态/第二遍扫描。
- [x] 2.3 增加必要永久fixtures、公开API和反例，并核停止。验收：固定完整方法实际恢复，构造/实参/branch全部physical BCI与UTF8来源合法；错误ownership/full conditional proof拒绝；真实late Stop dimension/usage/location、无当前方法部分产物，pre-cancel单列；旧New/field/conditional/loop回归通过。

## 3. 完整验收与主线交付

- [x] 3.1 冻结完整输入闭包的新CLI，实跑固定与控制双JDK完整三方并root独立接受。验收：所有成功生成类按原source完整重编执行、exit/stdout/stderr/对象身份同oracle；失败raw保留，JADX差异单列，default/all与方法成员身份/来源准确，不借旧CLI结果。
- [x] 3.2 完成fresh全工作区及必要lint/规范。验收：从target不存在开始按实际metadata全部目标恰好一次，named outcomes独立核；fmt/CI同范围Clippy/global OpenSpec strict通过，default Rust stack/5GiB free/target1GiB一秒守卫且结束clean。
- [x] 3.3 提交推送产品并接受其自身CI。验收：确切product head/run全部jobs/steps/双seed目标与本地同输入闭包，失败修复后用新产品自身CI，不借前片绿灯。
- [x] 3.4 更新verification/固定账本/handoff并交付干净main。验收：实际HEAD=main=origin/main、仅main、所有aux worktree detached干净main祖先无target/分支占用；整个CF11/71剩余范围和普通Site发布/foreach/generic独立债务明确，关闭docs推送后按真实HEAD最终audit。
