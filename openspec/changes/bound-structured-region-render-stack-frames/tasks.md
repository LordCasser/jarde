## 1. 根因和证据

- [x] 1.1 归档真实默认栈失败、8MiB诊断、LLDB尝试与最终调用栈及机器码frame。验收：精确绑定失败binary和完整编译源码closure，区分失败尝试，不把大栈成功当验收，不将binary本体写入仓库。

## 2. 最小私有重构与行为验收

- [x] 2.1 Luna准备Builder大变体私有抽取，root归一化逐句审查后应用，并撤回无必要的Region比较分支抽取。验收：公共declare_at一次执行；作用域、RegionPath、预算、checkpoint、状态与表达式顺序原样保持，无新类型或依赖。
- [x] 2.2 默认栈运行真实CF11完整类/API及晚Stop、预取消，并测新机器码frame。验收：无RUST_MIN_STACK覆盖，精确方法来源和完整BCI闭合；default/all相等；真实frame下降，不宣称跨平台统一栈预算。
- [x] 2.3 运行受影响循环、try/finally、return和有界恢复回归及完整Java库。验收：保留准确命令、退出值与失败原记录；消费和停止顺序没有行为回退。

## 3. 同版本交付

- [x] 3.1 执行fresh workspace、CI等价Clippy/fmt/OpenSpec strict和新CLI对照。验收：完整source closure恒同，5GiB/target1GiB一秒守卫且结束clean；可与CF11同版本检查共用实测，不能借旧产品结果。
- [x] 3.2 提交推送实际产品并核它自身CI与干净主线交接。验收：HEAD=main=origin/main及所有own CI jobs/steps核准；handoff区分本债务和未关闭CF11/71单元。
