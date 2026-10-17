# 父计划补齐工作流

先读actual status/apply与design instructions，再创建此前缺失的design.md，补齐instrumentation并协调proposal/spec/tasks。当前status仅表示planning artifacts完整；actual apply是4/7，三验收未完成，父项未同步或归档。两版validate --all --strict均363/0，1.14已有INFO按原始stderr保留。actual argv/cwd/exit和原始stdout/stderr相邻，未伪造额外产品运行。4945当前生产源码字节仍等native，见current-product-source-check；产品CI保持30afa本身。

后续独审已确认2.3/3.1/3.2在当前固定fixture范围证据充分，root据此勾选7/7。早先4/7收据保留为当时状态；最终status/apply及sync/archive/strict另以新label保存，不改写旧收据。JADX FX仍两次编译失败，返回覆盖/主动抛异常红项保留。

范围纠正：root重读完整EX发现finThrow/nestedFin仍为非覆盖型正向欠账，因此在任何main spec写入或archive之前撤销父整体收口。旧7/7及窄范围assessment保留但不作最终验收；当前4/7为1.1/1.2/2.2/2.3完成，2.1/3.1/3.2待办。后续使用scope-corrected新label，不覆盖旧原始收据。

original-scope-correction-review-v2已独立复核旧proposal原字节、完整EX当前报告及原类Code，确认保留其它未证明正向形合理，并撤回先前窄范围整体收口建议；相邻v1/v2与原窄assessment均原样保留。memo-v3的文件/Code/raw hash独审通过，引擎digest与SHA256分开。

scope快审指出仅异常完成缺正向scenario，已明确转后续独立变更；本父try-return仍待EX.finThrow/nestedFin，并为两者补窄正向scenario。当前没有同步、归档或新生产实现。
