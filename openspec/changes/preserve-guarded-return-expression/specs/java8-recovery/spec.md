## ADDED Requirements

### Requirement: Proven synchronized returns preserve their direct expression

当原 class 的返回值在已证明的 `synchronized` 正常退出前求值，且生成的 Java 把 `return` 写在同一同步块内时，系统 SHALL 在该返回位置呈现原表达式，不得只因同步退出指令存在而发明局部变量。返回表达式的求值次数、与释放监视器的先后、异常及来源 SHALL 与原 class 一致；缺少完整证明时 MUST 保持安全的现有呈现或明确拒绝。

#### Scenario: Field value is returned inside the proved monitor

- **WHEN** 同步块内的 `getfield` 产生返回值，正常路径先 `monitorexit` 再 `ireturn`，且退出处理器已完整证明
- **THEN** 源码 SHALL 在同步块内写 `return this.n;`，不得写块外第二次字段读取，也不得仅因退出指令而写 `saved` 局部；Java 8 重编后的值和异常 SHALL 与原 class 一致

#### Scenario: An independent effect separates the producer from the return

- **WHEN** 返回值生产后、同步退出前有不属于该返回表达式或同步语句自身清理的可观察指令
- **THEN** 系统 MUST 保留该效果与返回值原求值顺序；不能证明安全内联或保存时 MUST 引用相关字节码，MUST NOT 把生产者移到效果之后再计算

#### Scenario: Exit ownership is not proved

- **WHEN** 监视器退出、异常处理器或返回值来源缺少同一同步语句的完整证明
- **THEN** 系统 MUST NOT 将退出指令当成可忽略边界来发出直接返回表达式；现有 fallback、来源与预算停止 SHALL 保持完整
