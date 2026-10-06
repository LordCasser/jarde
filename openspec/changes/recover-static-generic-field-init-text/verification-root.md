# Root 独立验收（2026-10-06，合并）

## 判据逐项

1. **落点更正（偏差 1）追认**：真根因在 `src/facade.rs::project_static_fold_owner_texts`（~21994）——静态成员折叠的字段重拼循环对**原地改写中的文本**按原始字节跨度 `replace_range`，第二处起每次替换晚 `prefix_delta` 字节吃掉名字后文本（`MN$Hold` 比 `Hold` 短 3 字节 → 吃掉 `((j`）。实现片以逐字节模拟复现巡查样本定案；同函数三个姊妹循环（22222/22517/23348）自后向前改写故不受累——与"只有此循环损坏"一致。原 spec 的"拒绝回退拼接点"前提作废，correction 记入 proposal 影响面。
2. **diff 审查**：1 文件 13+/3-——每处编辑跨度按累计增量平移（`local_start/local_end`），与 derived 记录已有的同一 delta 纪律；无平行通道。
3. **root 实测（合并态自建 CLI，jar 输入+自述头断言）**：MN/RG/SG/P1 四锚 `Holdava=0`（损坏文本绝迹）；`recover_static_generic_field_init_text` 3 passed + 1 ignored 回放（SG/P1 编译 exit 0、运行与原一致 `ab5`/`1`）。
4. **验收条款更正（偏差 2）裁定**：MN/RG 整类 `javac --release 8` exit 0 **不可达属既有独立债务**——修复后其唯一残余错误是嵌套 `Hold<T>` 自身的擦除对投影（`T v;` 与拒绝态 `Hold(java.lang.Object)` 并存），单类呈现同在、投影域明确出本片范围。**接受条款修订**：本片判据=损坏文本归零（已证）+ 可达编译锚（SG/P1）绿（已证）；`Hold<T>` 擦除对投影登记为投影域后续项，不入本片。
5. **门禁**：全量 **3104/0/61**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **303/303**。corpus 差分（自检先行+假零自纠记录在案）：moved=10（巡查 MN/RG + 4 fixture ×2 腿），全部为字段声明行。
6. **flake 复跑（并入本批）**：`d3_artifact_binding::the_evidence_is_rebuilt…` 本地两轮绿——注意**须 `--all-features`** 才与 CI 同口径（裸跑该 target 为 0 测试，是一次假零口径陷阱）；家族判定三连齐（docs-only 红 dccd21c3 / 同代码绿 b6cbaa94 / 本地两轮绿）。
