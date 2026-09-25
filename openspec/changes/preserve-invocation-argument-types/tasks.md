## 1. 固定重载及类型边界

- [x] 1.1 收成独立 Java 8 fixture 和调用恢复测试，覆盖 Object/null/数组/装箱、数值加宽、byte/short 常量、构造器、多参数、外部泛型 helper、函数式目标；保留三方执行基线及合法无 checkcast 接口反例，至少直接重载错值测试修前失败。不得用删改恢复方法体掩盖 lambda synthetic 冲突。

## 2. 按调用位置保留参数类型

- [x] 2.1 修正既有parameter_types对引用/数组退化为Object的历史boolean专用占位，以现有描述符类型读取保留真实presented及slot布局；不在调用层以SSA类型覆盖源码静态类型。使用既有 Cast/Type 和 descriptor 构造安全的调用参数类型；同型稳定值不重复包装，poly Call 即使同型也限定。未知引用关系和参数事实缺失均拒绝。验证 Object/数值/窄常量/未知接口的正反例，赋值/返回/concat 回归不变。
- [x] 2.2 从已证明的函数式工厂返回类型携带类型事实，在外层调用实参处固定该目标；同时覆盖 Object 消费、new 与 this/super 参数入口。来源使用已有原始及 derived 锚点，拒绝保留延期效果，验证 source map 与生产者次数。

## 3. 执行与独立验收

- [x] 3.1 用真实恢复正文重编译，对照原 class 的重载返回值、多参数顺序、null、计数与异常；外部泛型 helper 保留原始声明，不能靠擦除 helper 泛型获得假通过。比较结果与 jadx 差异写入 verification.md。
- [x] 3.2 主代理审查、独立样例执行，复跑转换、构造器、lambda、bridge、旧值、预算/失败保留回归；只调整由本项规则解释的旧调用文本断言，集中更新 census/fingerprint，执行 fmt/clippy/OpenSpec strict，并分别记录新回归与既有门禁债务。
