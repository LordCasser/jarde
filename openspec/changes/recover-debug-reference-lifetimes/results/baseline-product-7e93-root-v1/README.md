# 当前完整类基线

产品 `7e93eb30647338dd8a6ddfc9994392f23457aa3e`，v12 CLI SHA `c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd`。真实-g与-g:none两次各25命令，共50命令/100raw、8份11方法Jarde报告、四个原类运行各17场景同原。JADX两rename profile均因raw alias消失而完整类编译失败。Jarde debug有String→Integer身份错误与handled stub；none只剩handled stub；完整类均compile fail/no-runtime。

Luna审计脚本 SHA `d5f81167e77f08f91410e1690fcfed22d6423ca1e89971cac415553622507f46`，root实际重放并得到逐字同audit JSON SHA `6948f44e342efff64c17c79b7f9330ff38bc3a8b7dea2bf458809b187b2563fe`。核全部raw/source/tool pins、CLI0555、真实compiler flags、17oracle、default/all全文恒同、每条combined stderr与handled uncovered[7,16,47]。报告digest是BLAKE3，不能与receipt SHA256直接比较；审计核内部snapshot/owner/member/来源一致，并独立核原class SHA256与长度。single-target isolated compile与combined输入重复，仅计8个独立Jarde编译场景。

capture.tar.gz为1082919B，SHA `b452c5251bb40bc750f553876a7afa3bed929b61fa53ababd7c9b05a34404bd1`。154证据成员加嵌入capture-manifest.json，实际155文件；root独立重新打开全部成员、核100raw及audit SHA。工具打印members=154指不含manifest，不能误写总155为154。旁侧pins核archive与manifest。归档脚本 SHA `8b4618df2eca44396f76c7bf14f9fde4bb8190006f060116e58f05b36fbf476f`，未封装CLI/JDK/JADX二进制，只保留工具引用与实际build receipt。

该基线只证明问题与验收边界，不能当候选实现成功。外层try的同次定位见[独立观察](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-13/outer-try-foreach-trace-root-v1/README.md)。
