## 1. 固定尾部完成与控制流边界

- [x] 1.1 复用既有static_initializer_class真实class生成器补初始化回归，保留source-only Java及runner；直线、条件、循环、异常、空块和合法提前返回变体在临时目录审计，不新增永久class。记录javap、原class/jarde/jadx实际编译执行，逐例区分return与其它既有拒绝；Rust正面测试修前应因非法尾return失败。

## 2. 统一正文发射

- [x] 2.1 在现有emitter共用body入口中实现静态初始化顶层末尾无值return的正常闭合；不改AST/区域/CLI，测试普通void、构造器、无声明及嵌套return均不被误消除，空块content保持实际语句数。
- [x] 2.2 沿用被投影return的来源在真实闭合字符生成映射；验证对应BCI与成员、commit/replay一致、默认不构造来源以及输出/来源预算停止契约。

## 3. 执行与独立验收

- [x] 3.1 将实际恢复整类重编译，对照已可恢复的直线/条件/循环/异常helper样例的初始化结果、调用次序、首次初始化异常和再次访问行为；不得删改输出或替换失败方法体，记录明确未覆盖场景。
- [x] 3.2 主代理复核实现与独立样例，复跑发射/声明/class-source及预算回归，集中验census/fingerprint、fmt/clippy/OpenSpec strict，按实测分别记录新增回归和既存门禁债务。
