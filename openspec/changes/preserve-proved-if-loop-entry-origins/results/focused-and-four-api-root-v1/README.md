# 定向与四API证据

capture.tar.gz收录100个payload及ARCHIVE_INVENTORY.json，inventory.json逐项记录原路径/bytes/SHA256/mode，root-review.json记录逐字复开。原始v1编译失败、v2断言失败和v3/v4续跑均保留；format/v1是测试尚未集成时的前置失败草稿，不当作成功formatter结果。两份同字节冻结CLI只在库存记录pin，不加入Git归档。

focused/v4是最终成功构建与公共proof-site停止收据；focused/v3提供508 Java lib、Reader、指纹和完整Clippy成功。four-api包括四实际CLI调用/原始stdout与完整JSON/独立review/root review。最终v4二进制字节等于four-api使用的v3；编译证据复用边界见上级README。implementation/v1保留root拒绝的额外CompoundIf/缺clone门初稿，v2为审后窄实现，shared后续只有rustfmt及测试修正。

本归档不含尚未发生的fresh workspace或自身CI，不声称目标执行等价。
