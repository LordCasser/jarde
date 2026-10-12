# 首轮全工作区的夹具登记失败

fresh workspace root-v1 实际退出1，第一seed的第64批（command index65）在 p5_corpus_fingerprint 报 corpus_files_match_the_recorded_fingerprint FAILED：本片4个永久fixture目录的30个意图保留文件尚未登记。该批实际5/0/1、29/0/1、4/1/1三个目标运行摘要，第四目标已编译但Cargo因前一目标失败未运行，因此记录器随后报 summary/artifact cardinality mismatch。它不是仅有记录器错误，更不能当整批通过。

先前63个批次的结果保留，但全workspace未完成，第二seed尚未执行，不与后续运行拼接。实际67条命令（含fmt、metadata、64测试批、clean）；失败后finally clean实际exit0、target absent、完整4822文件source closure一致，没有资源guard中止。修正方向是审查本片冻结fixture来源后仅登记这30项，不全量重写既有清单或改变门槛。

capture/tooling档保留实际receipt、所有raw输出与当时runner。单独保存65stdout/stderr便于核具体失败和逐项BLAKE3/bytes。后续永久unit和fixture登记后须fresh来源闭包与完整门禁，不借用此轮partial结果关闭task3.2。
