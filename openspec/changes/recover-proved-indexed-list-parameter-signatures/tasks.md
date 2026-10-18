## 1. Baseline and architecture

- [x] 1.1 root实际冻结完整上游TestNestedLoops2与独立控制，验收源码/原始class/JADX/Jarde的两编译器、双VM、size/get事件/异常、CodeSource正负控制和全部Code/BCI/原始流；分列JDK8旧框架边界和所有真实失败，准备或静态审查不算完成。
- [x] 1.2 root接受完整体结果类别、List.get物理/AST闭包与最小opaque查询的对抗审查；proposal/spec/design/tasks全部ready且strict通过，并按本change实际派发Luna私有实现，保存派发/范围记录。

## 2. Implementation and regression

- [x] 2.1 Luna私有实现复用参数census、精确primitive/void carrier、get→真实cast(E)/AST关系及stage/final重验证；root审查集成后运行focused证明/公开测试，验证固定Boolean、void、int/long直接消费者及既有泛型返回分支，真实cast不删除。
- [x] 2.2 root实际运行错primitive/Signature、未知调用/错cast/额外用途/别名/替换Phi/重复或缺AST、late预算/取消/期限和default/all/essential边界；相关泛型、foreach、引用/数组/方法类型变量回归及fmt/CI Clippy/MSRV适用广门全过，绑定不变来源和最终CLI。

## 3. Complete comparison and delivery

- [x] 3.1 root用最终CLI对完整固定原始及独立控制做源码/JADX/Jarde双编译器/双VM对照，核精确List<E>与primitive结果声明、保留cast的Code/事件/异常、raw/map平面和错根控制；不借原始target运行生成类，不把编译失败当runtime通过。
- [x] 3.2 root完成fresh workspace、提交推送产品并独立验收其自身CI全部jobs/官方step及两seed/新增具名门，绑定来源/CLI/Git；同步主规范、固定账本与handoff，strict/apply通过，target清空且main干净、与远端一致后关闭本片。
