Root修正v5实际失败：no-clinit manifest source_files路径相对no-clinit-super-args-v1源目录，而非baseline-root-v2。只改变source记录的读取base，不放松bytes/SHA/pin或闭合基线核对。v5实际stderr保留；v6执行状态由独立execution记录确认。
