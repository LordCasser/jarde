# Root 静态审查与应用

v2 基线与候选字节 pin 全部一致，1.2 真实控制基线接受后应用三个 source 文件。v1 普通构造二次扫描/重复计费及额外 refusal Vec 排序已由 v2 消除。one-hop 只给 Builder 选择候选，不建立 CFG 或移动 effect。最终 AST 一次扫描，direct method-local New 计数决定同一 summary/refusal/record。

Root 修正返回 iterator 的 Rust lifetime：self 与 immutable committed 共享显式 `'a`，filter closure `move` capture reference，避免返回借用本地 parameter。测试及新 CLI 均尚未执行，不接受产品恢复。shared/nested 等边界缺实际永久测试，不能将唯一wrong-block unit当全部否例。
