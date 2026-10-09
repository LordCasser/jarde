## ADDED Requirements

### Requirement: 同类泛型调用消费位的完整源码证明

系统 SHALL 对已完成正文、声明作用域与物理擦除一致的同类普通调用，联合证明实际源码 receiver、逐个实参及结果消费，恢复可闭合的 caller 泛型声明。声明中的类型变量 MUST 保持其类或物理方法 binder 身份；仅相同名称或擦除类型 MUST NOT 作为赋值与推断依据。支持范围 SHALL 包含直接类型变量、变量数组、已验证上界、多参数及宽槽、直接方法变量代换和有限无环 relay 链。

#### Scenario: 直接参数与调用返回
- **WHEN** CallRelay<T> 的 identity(T) 返回原参数，而 relay(T) 返回该调用结果，参数未改写且所有调用及结果使用均获证明
- **THEN** 两个方法 SHALL 保留 T 参数与 T 返回类型；完整类 SHALL 重编并返回同一 marker，反射中的变量 SHALL 指向原类声明

#### Scenario: 空体与 void 转发
- **WHEN** EmptySink<T> 的 sink(T) 不使用参数，或 VoidDirect<T> 将原 T 参数转发给同类 void sink(T)，其余消费位完整可证明
- **THEN** 系统 SHALL 恢复两种 T 参数声明并保持调用次数和效果；不能以空体或 void 为由省略使用证明

#### Scenario: 数组与多参数槽
- **WHEN** 合法 relay 将 T[]、T extends Number 或多个类变量按对应位置传给同类方法，且 long/double 参数使后续槽位跨越两个物理槽
- **THEN** 系统 SHALL 保持数组维数、组件 binder、边界和每个真实参数位置，整类重编、marker/数组身份及泛型反射 SHALL 与原类一致

#### Scenario: 直接方法 binder 替换与遮蔽
- **WHEN** 已获证的 <U> U identity(U) 或 <U extends Number> U identity(U) 接收 caller 的已获证直接变量，全部相关实参与返回约束有唯一一致替换，包括 caller 的方法 T 遮蔽类 T
- **THEN** 调用 SHALL 保持推断或有证据的显式类型形态，caller 声明 SHALL 恢复；类 T、caller 方法 T 与 callee U MUST 保留独立 GenericDeclaration 身份

#### Scenario: 有限链不依赖物理声明顺序
- **WHEN** 同类多个无环 relay 依赖同一个已证明 identity，classfile 声明顺序与调用依赖顺序不同
- **THEN** 系统 SHALL 恢复完整有限调用链，并保持所有结果身份；MUST NOT 仅恢复扫描顺序碰巧先见的第一层

### Requirement: 调用与泛型声明的关联原子发布

系统 SHALL 在最终实际声明和正文上验证所有相关同类入边、实参和结果消费后，一起发布有关联的源码改变。未知、被拒绝或未完成证明的 Signature MUST NOT 冒充实际类型；不能通过一律擦除独立已证 callee 或给 Object 强转到未知 T 来掩盖恢复缺口。不能闭合的关联集合 SHALL 保留可靠的擦除表示与具体拒绝证据，不留下部分声明造成的新编译错误。

#### Scenario: 不兼容或未知 incoming caller
- **WHEN** 一个候选 relay(T) 同时有安全 caller 和无法适配的 Object caller，或者字段/独立方法变量与结果约束不相容
- **THEN** 未证明入边 MUST NOT 被忽略；相关声明和正文 SHALL 一起拒绝或获得完整类型证明，保留真实物理调用与效果，不按相同擦除合并 binder

#### Scenario: 非调用关联的既有泛型头
- **WHEN** 同一类中有与失败调用集合互不依赖且已独立获证的泛型成员
- **THEN** 该成员的合法投影 SHALL 保留；MUST NOT 一律擦除整类所有方法作为失败调用的处理方式

#### Scenario: Raw receiver 与未知代换
- **WHEN** 实际 emitted receiver 是 raw 类、保留 raw local 或发生重绑定，或者调用需要未知嵌套/wildcard/跨类泛型代换
- **THEN** 系统 SHALL 使用可证明的真实选择类型或可靠拒绝；raw receiver MUST NOT 自动视作 this/参数化 receiver，参数化 receiver MUST NOT 自动视作 raw；未知 Object 返回 MUST NOT 冒充 T

#### Scenario: 停止与不完整清单
- **WHEN** 采集、作用域代换、依赖遍历、入边验证、AST 投影或文本发布遇到预算、取消或不完整清单
- **THEN** 停止状态 SHALL 传播，相关源码提交 SHALL 原子中止，未完成 callee 类型 MUST NOT 进入后续字段证明

#### Scenario: 物理调用存在而候选清单遗漏
- **WHEN** 关联 caller 的完整物理调用事实含两个同类 invoke，而候选清单遗漏其中一个或全部，或者已提交物理调用在最终 AST 无唯一匹配节点
- **THEN** 系统 SHALL 以现有 Code/CP 清单和同次 retained AST 调用目标精确交叉核对，拒绝受影响的关联投影；无关已证成员 SHALL 保留，缺项 MUST NOT 因剩余 site 都匹配而被视作完整

### Requirement: 已恢复构造正文中的泛型调用适配

系统 SHALL 对 Object() 初始化后已完整恢复的调用结果赋值构造器，在独立证明全部参数使用、调用类型和初始化次序后恢复类变量参数。普通已结构化 catch SHALL 按真实异常边保留，不能仅因存在 EH 而丢弃可闭合的调用适配，也不能把现有直接字段构造器证据冒充调用/EH 证据。字段投影 SHALL 继续在方法声明最终发布之后独立获证。

#### Scenario: CallHold 与 ExceptionHold
- **WHEN** CallHold<T> 或 ExceptionHold<T> 在 Object() 后把 identity(v) 存到字段，前者为直线正文，后者为已结构化普通 catch，所有涉及 T 的调用与使用可闭合
- **THEN** constructor 参数 SHALL 恢复为原类 T，整类 SHALL 在四条冻结腿重编并保持 marker 和异常行为；字段仍 SHALL 依据其全部真实写入消费独立决定投影，不能把字段反射缺口误报为方法失败

#### Scenario: 初始化和异常语义不因适配改写
- **WHEN** 相同 constructor 存在调用可抛异常、catch 中字段写入或非调用副作用
- **THEN** 初始化、调用次数、写入顺序、handler 和抛出行为 SHALL 保持；this/super 委派或未证明异常区域 SHALL 保留本片之外的可靠拒绝

### Requirement: 泛型头恢复后的源码重载目标保持

系统 SHALL 依据实际源码实参类型、封闭可见重载集合和原物理 invoke 目标验证 Java 源选择。必要的转换 SHALL 仅在已证明安全、不会增加运行时类型检查且能够排除竞争目标时发射；MUST NOT 以物理 descriptor 正确替代源重载证明，MUST NOT 引入未证明的向下转型、装箱、varargs 或副作用。

#### Scenario: Intersection bound 的丢失上转型
- **WHEN** BoundOverload<T extends Number & Comparable<T>> 的 relay(T) 原物理调用为 pick(Number)，直接 pick(v) 在实际源重载集合中有歧义，而 (Number)v 是可证明的无运行时检查上转型并唯一选择原目标
- **THEN** 输出 SHALL 保留 Number 目标选择，完整类 SHALL 重编并打印 number；JADX 同例失败 SHALL 作为对照事实保留而不能作为 Jarde 的通过替代

#### Scenario: 不需要转换的唯一调用
- **WHEN** 已获证的泛型调用在实际源类型下唯一绑定到原目标
- **THEN** 系统 SHALL 保持直接调用，不添加无用 cast，也不以该成功推定其它重载集合已被证明

#### Scenario: 相关重载的未读参数声明
- **WHEN** 同一封闭 overload 集中一个成员的 Signature 明确包含 Comparable<T> 参数，完整同次 Code/SSA/effects 证明该 physical formal 未被正文读取，正文其余效果已完整恢复
- **THEN** 系统 SHALL 按该成员自身 Signature、擦除和已发布 binder scope 恢复参数声明，再独立验证调用方的完整 overload 选择；MUST NOT 把参数未写或不完整清单中的零读数当作未读证明

#### Scenario: 无法封闭的重载与来源
- **WHEN** overload 集、目标身份、AST 实参来源或实际类型不完整，或所需 cast 会增加运行时检查/改变选定目标
- **THEN** 相应投影 SHALL 可靠拒绝并指出调用位置；MUST NOT 用猜 cast 生成表面可编译但行为不同的源代码
