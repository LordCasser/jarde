## 1. 关系与类型作用域

- [x] 1.1 复核现有家族 prepared/capture/call gate、reader Signature 作用域与选定成员路径；为唯一泛型根/直接 child 定义物理身份、根 T 与方法擦除的不变量
- [x] 1.2 在家族关系证明之后，把已证根 T scope 提供给 child 方法 Signature 与根 `Outer<T>.Inner` 返回路径；不影响独立 child 请求或非泛型家族

## 2. 原子家族投影

- [x] 2.1 复用现有 constructor/capture 证明和结构化 writer，一次输出嵌套 `Inner`、`T id(T)` 与 `Outer<T>.Inner make()`，保留双 owner 物理报告/source map
- [x] 2.2 对不支持的根作用域/擦除、缺失构造证明、静态或额外分配形状、第二 child、effectful child 方法和预算/取消进行有证据等级的拒绝测试；任何未闭合方法不产生半个源码家族

## 3. 三方验收

- [x] 3.1 扩展 DT-19 replay 修后模式，原/JADX/Jarde 全源码与同一个 `Outer<String>.Inner` consumer Java 8 重编、`-Xverify:all` 运行一致，保留 raw/非泛型邻接控制
- [x] 3.2 运行相关 Rust 回归、fmt/check、OpenSpec strict 并清理隔离 Cargo target；多 child 和无 debug 局部泛型流继续独立审计
