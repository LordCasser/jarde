## 1. 加载位置模型

- [x] 1.1 将 LoadRoot 收敛为 standalone、container+raw prefix、external，一次迁移 environment/view identity、provider 映射、snapshot 提取、CLI JSON 和仓库调用点；以 workspace 编译、序列化 round-trip、未知/旧变体拒绝及不同 prefix 的身份区分验证 breaking 迁移。
- [x] 1.2 在统一候选查找和 caller/driver binding 中使用精确 prefix 拼接，校验 Header 内部名一致；以无目录 entry 的 WAR、空/非法 prefix、原始字节、大小写、点段及 name mismatch 验证，不通过改写物理名或跳过 binding 获得成功（A07）。
- [x] 1.3 验证 ParentFirst/ChildFirst、root 顺序、应用目录与 nested lib 同名、同位置重复、相同内容不同 origin、坏候选和不完整目录；确认改变 prefix 后不复用旧 unbound/missing，cache 可用时增加 off/on 一致性对照（A07/A14/A15）。

## 2. CLI 最小闭环

- [x] 2.1 增加薄的 enumerate_artifact_tree JSON operation，直接返回库 report；验证普通 enumerate 不递归、tree 的 Partial/Cancelled/坏 child 与库一致，且不自动生成 roots 或改写 snapshot（A08/A14）。
- [x] 2.2 用受控 WAR 完成“CLI 枚举→真实身份与显式 prefix→recover_method”闭环，并与同输入/预算的库结果比较，source map 保留 WEB-INF/classes 物理路径；另用 BOOT-INF/classes 受控样本证明泛化 prefix，不宣称完整 Boot loader 支持（A07/A08/A16）。

## 3. 文档与验收

- [x] 3.1 更新 root API/JSON 示例与支持边界，运行 reader/resolver/runtime-view/CLI 相关测试、R8/R9 和 fmt/clippy/OpenSpec strict；记录固定 SHA 与结果，确认 MR/module/自动布局加载支持范围未被扩大，验证通过后同步/归档。
