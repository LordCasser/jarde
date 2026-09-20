## ADDED Requirements

### Requirement: Driver class declarations use the already-read physical evidence

公开恢复入口 SHALL 向声明恢复提供本次 driver Header 已取得的 class internal name 和 class access flags，并绑定该读取的物理定义；MUST NOT 要求调用方再次读取或手工补充这些已有事实。事实交接 SHALL 不新增 Header/Body 扫描，也不从 entry 显示名、CP 引用或宿主 classpath 推断声明类。名称的安全显示不能改变原始身份。

#### Scenario: Ordinary and interface method declarations

- **WHEN** 公开入口分析普通类实例/static 方法及 interface default/static 方法，相关 Header 已完整读取
- **THEN** 声明记录基于实际 class/member flags 给出对应 form，不产生因交接丢失导致的 class-not-in-run 诊断；正常方法不额外读取其它 Header 或 Body

#### Scenario: Same name with different physical declarations

- **WHEN** 两个物理定义或 loader 下存在相同内部名但 class flags 不同的类
- **THEN** 每个恢复请求使用其实际绑定定义的 class facts，不因同名或相同显示文本串用声明

#### Scenario: No declaration was published

- **WHEN** 本次分析在取得可靠 driver 声明前停止，或低层恢复输入本来缺少声明类事实
- **THEN** 继续报告缺失事实与原停止语义，不通过请求参数猜测 flags 或伪造 body

#### Scenario: Declaration evidence is not a body quality claim

- **WHEN** 声明类事实交接成功，但方法内部某个结构仍无可靠恢复证据
- **THEN** 声明可正确呈现，方法继续保留原 fallback/未证明状态，不仅因声明诊断消失升级 body quality、编译或验证标志
