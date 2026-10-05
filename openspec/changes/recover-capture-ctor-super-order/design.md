# Design：捕获形伴生 ctor 的 super 重排

## Context（root 已实测）

- 字节码：`aload_0; aload_1; putfield val$s; aload_0; invokespecial ArrayList.<init>` —— val$ 赋值先于 super（javac8 捕获惯例）。
- 渲染现状逐字跟随字节码次序 → 源级非法（super 前 this）。
- 无捕获形健康；jadx 以分配点双括号形彻底解决（本片取最小重排路径）。

## 决策 1：重排条件 = pre-super 全捕获赋值 + super 实参无依赖

可证安全当且仅当：
1. super() 调用之前的语句**全部**形如 `this.val$x = argN`（捕获字段赋值，无其它副作用）；
2. super() 的实参（若有）**不读取**任何被赋值的 val$x（数据流判定——实参只依赖参数/常量/this 外的值时成立）。

满足则重排：`super(); this.val$x = argN; …`。任一不满足 → 保持现状（渲染头已声明 not claimed to compile，响亮）。

## 决策 2：呈现次序与源形

重排后 val$ 赋值组保持原相对次序，置于 super() 之后、实例块语句之前（与 javac 的源级匿名类语义一致：捕获字段先初始化再跑实例块）。

## 决策 3：零回退与负例

- 无捕获形（`DB$1`）伴生逐字节不变；
- 宿主呈现（`new DB$2(arg0)` 调用形）零改动；
- 负例：super 实参依赖捕获值的合成形（如匿名类 `extends Base` 且 `super(s)` 用捕获 s——手工字节码或混淆产物）保持现状；
- corpus 双腿扫描：差异类仅为捕获形伴生（如实记录数量）。

## 验证标准（可证伪）

1. 主锚：`DB` 双形拼接 `javac --release 8` exit 0（修复前 exit 1）、行为 `2/z` 逐行一致；
2. 零回退/负例如上；corpus 差异边界记录；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 伴生 ctor 渲染的落点（member_inner vs facade——task 1.1 定位 val$/super 次序逻辑所在）；
2. 真实语料捕获形伴生数量（corpus 扫描给出——决定该缺陷的暴露面）。
