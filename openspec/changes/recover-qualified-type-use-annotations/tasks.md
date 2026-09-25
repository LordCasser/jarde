## 1. 冻结三方差异与源码位置边界

- [x] 1.1 `type-use-annotations/` 冻结原 class/JADX/Jarde 的完整源码、属性与执行；root 在独立目录重放 summary 字节相同，原/JADX 全套 Java8 编译及验证分别为 `field/return/parameter` 与三行 `null`，Jarde 全套因 runner 缺返回失败，隔离 subject 可编译并复现三行 `null`。
- [x] 1.2 `placement-boundaries/` 冻结双目标注解普通前缀与限定名内部位置，root 在 `/tmp/jarde-type-place-root-MrdObq` 重放 summary 字节相同，`javap` 与反射均证实普通前缀双属性、限定名内部只含类型属性。
- [x] 1.3 `primitive-boundaries/` 冻结双目标 `@A int` 的字段/返回/参数及仅移除三处声明属性的受控 classfile；root 在 `/tmp/jarde-type-primitive-root-LUoXSh` 独立编译、patch、`javap -v -p` 和 `-Xverify:all`，两份 class SHA 与记录一致，三处声明反射均空而类型反射均为 `[A]`，三个完整 Code 属性哈希补丁前后相同。

## 2. 复用属性树并在可证明的类型位置拼写

- [x] 2.1 在 `recover-member-annotation-uses` root 验收后，扩展 reader 的 `RuntimeVisibleTypeAnnotations`/`RuntimeInvisibleTypeAnnotations` 读取：保留每项完整 target/path/annotation、精确属性末端及物理次序；reader 正负、预算、取消和无壳零内容读取测试通过。
- [x] 2.2 把字段/方法自身类型属性从同次成员读取交给类源码，JSON 并列原 shell、target/path、值、拼写或拒绝；owner 错置与非本轮目标不误写，字段/方法原声明注解和正文回归通过。
- [x] 2.3 对空路径的非数组限定引用类型，在字段、返回和 descriptor 形参位置使用限定名内部语法拼写；测试 Java8 重编译、`-Xverify:all`、三个 `AnnotatedType` 值与原 class 一致且没有新增声明注解，并用宽槽参数验证位置。
- [x] 2.4 对不可见属性以 classfile owner/target/value 对照验证；对基本类型、单段名、数组/非空路径、重复及跨平面同类型、越界参数和不可拼写值作原子拒绝或真实 stop，验证壳与原因可见、其它独立注解仍可读、无伪造属性。

## 3. 独立执行与主代理验收

- [x] 3.1 用修后冻结 CLI 原样重放 1.1–1.3，记录完整三方编译状态与隔离 subject 的 Java8 编译/`-Xverify:all`/反射、不可见属性和双目标边界；不手改生成源码，不把隔离执行称为整类通过。
- [x] 3.2 root 审查 reader 目标格式、归属、源位置歧义与预算/停止，独立重建 CLI 并从复制目录重放全部证据；定向 Rust/Java、reader census/fingerprint、fmt、Clippy 和 OpenSpec strict 验收通过，独立架构债务另列。最终 CLI SHA-256 `d7520a08a37b54ef579d42c770f5b512af17ddb11b05050568458759c78dbf16`，见 `verification.md`。
