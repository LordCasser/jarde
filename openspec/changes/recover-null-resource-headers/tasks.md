## 1. 冻结最小证据

- [x] 1.1 把 `try-with-resources/null-resource/` 的 Java 8 源、670B原 class、runner 与 javap 关键 BCI 固定为永久夹具；用原 class/JADX整类重编译和 `-Xverify:all` 行为一致、修前 jarde 全类 javac 失败作 RED 验证，不改反编译文本。
- [x] 1.2 加同一类型下普通 null 赋值后用户 catch 的正面分类与 close/抑制近似形状负面夹具；验证原/变体 class 的真实字节码、可验证性与改前分类，保留 Code/方法哈希及来源以排除无效输入。

## 2. 复用资源证明闭合 null 头

- [x] 2.1 在现有 guard 候选过滤里只接受唯一 `aconst_null; astore` 且具有资源关闭轮廓的头，随后复用原 TWR 全证明；用 null 正例、普通 catch 正例、损坏 close/抑制/重抛负例及既有 `p3_guard` 断言结构与引用边界。
- [x] 2.2 在现有资源计划中携带必要的异常 close 锚点；仅当正常/异常 close 属主都是可拼写当前类，且本类直接声明 `AutoCloseable` 时将 null 声明写成当前类类型，其它情况有来源地拒绝；用 null class 完整源码 `javac` 与缺类型负例验证，不改非 null 类型推断。
- [x] 2.3 核验 null 头/正常与异常 close/抑制的 source map、默认与 all 正文、预算与取消；用永久 Rust 测试和相邻 TWR/typed catch 测试证明调用、异常及引用不丢失。

## 3. 整类验收

- [x] 3.1 代理原样重编译并执行 null/core/multi-resource 三类的完整 jarde 输出，逐行比对原 class/JADX 的 `-Xverify:all` 结果；另测 null 资源正文抛异常时 primary 对象身份、suppressed 为空和 close 不运行。null 正例零引用且无合成用户 catch；记录脚本、CLI SHA 与未覆盖边界。最终冻结CLI `30481c14…` 的root独立六组重放见 `root-after-null/`，JADX异常正文编译失败单列。
- [x] 3.2 root 独立重放三方整类、审读 guard 分类与资源类型约束，复跑 TWR、普通 catch 和资源来源相邻回归；把任何独立债务记在 survey 而不混入本 change。普通null局部类型与抑制拒绝文案另记，不扩当前范围。
- [x] 3.3 root 完成 reader census/fingerprint、`cargo fmt --all -- --check`、相关 Cargo 检查及 `openspec validate --all --strict --no-interactive`，将实际通过/既存失败写入 verification 后勾选。census 111/726/94/261/8、指纹293、strict54/54通过；严格Clippy只剩既存区域类型复杂度警告。
