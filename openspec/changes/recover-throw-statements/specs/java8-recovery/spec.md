## ADDED Requirements

### Requirement: Ordinary throw statements preserve exception evaluation and identity

对于已接受区域中可忠实呈现的异常表达式，恢复结果SHALL包含相应throw语句，保持原对象身份、null异常语义和表达式的单次求值。恢复器MUST NOT用返回、独立调用或新异常对象替代原来的throw。

#### Scenario: Direct exceptions and null
- **WHEN**原方法直接抛出参数异常或null
- **THEN**恢复正文分别抛出同一参数对象或null，实际执行的异常类型和身份与原class一致

#### Scenario: Constructed and returned exceptions
- **WHEN**异常来自可呈现的构造表达式或调用结果
- **THEN**异常生产者只在原位置执行一次，其自身抛错仍先于外层throw，调用计数与原class一致

#### Scenario: A cast precedes the throw
- **WHEN**原方法显式检查异常对象的引用类型后抛出
- **THEN**恢复正文保留该检查，兼容对象保持身份，不兼容对象先产生ClassCastException，null仍产生NullPointerException

#### Scenario: Existing branches and catches contain a throw
- **WHEN**throw位于已经可恢复的条件分支或命名catch保护区域中
- **THEN**恢复正文保持原来的路径选择与捕获行为，不因省略throw而使原来的终止路径正常完成

#### Scenario: Checked exception declarations are already available
- **WHEN**原方法抛出checked exception且自身声明已有对应throws事实
- **THEN**恢复的throw与原有throws声明共同保留，不伪造新的异常类型或丢掉该声明

### Requirement: Throw recovery retains producer ownership and bounded evidence

throw呈现SHALL沿用现有求值、效果、来源与预算契约。无法忠实呈现的消费MUST保留其异常生产者和throw来源，不得因延期而丢失原程序效果；恢复器MUST NOT将已有形状持有的合成重抛重复输出，也不得把源码恢复结果标记为已完成JVM验证。

#### Scenario: Unsupported or stale exception expression
- **WHEN**异常表达式无法呈现、经过不支持的重复消费或在呈现位置已不再指向原值
- **THEN**相应结果明确拒绝并引用相关生产者与throw，不搬移写入、不重复求值，也不凭空插入Throwable强转

#### Scenario: Lower stack values are discarded
- **WHEN**合法class在throw执行时还持有更低的栈值且其生产过程有副作用
- **THEN**恢复结果继续保留已经执行的副作用，不把该栈值误作异常表达式，也不因终止清栈而删除它

#### Scenario: Guarded synthetic rethrow ownership
- **WHEN**原有异常或同步形状已持有某个合成重抛指令
- **THEN**普通throw恢复不再次输出该指令；不在本项中放宽原本被拒绝的finally或synchronized区域

#### Scenario: Source selection and budget stop
- **WHEN**请求throw正文的来源证据或预算在正文及证据阶段停止
- **THEN**已交付来源关联真实throw及其表达式的BCI和成员，默认未请求的来源不构造，停止契约及已提交正文保持一致
