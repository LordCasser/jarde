# nestedFin 产品自身 CI 独立验收

产品 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5` 的 [CI39042611463](https://github.com/LordCasser/jarde/actions/runs/39042611463) 由 root 实际收集并独立审读。4 jobs / 52 API steps 全成功；两 seed 各376 targets、3691通过、0失败、98忽略、0 filtered，14项关键门均各一次通过。MSRV1.88、JDK25额外对照、依赖层级、双workspace supply chain和3项fuzz smoke通过。

6收集命令/12raw流逐字核大小与SHA；workflow与当前源码绑定产品。capture保留API、四job完整日志、workflow、收集器与独立接受器及其实际输出。此收据属于产品提交，不能冒称后续文档head自身CI。
