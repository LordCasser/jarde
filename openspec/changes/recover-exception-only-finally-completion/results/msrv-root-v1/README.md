# 最终源码 MSRV 1.88

实际rustup run 1.88.0 cargo check --workspace --all-targets --locked通过；version/check/exact clean三命令六raw逐项核。原Cargo直接+1.88 launcher已知不支持，因此使用rustup入口，无新增依赖。默认栈/5GiB/1GiB/每秒守卫通过，来源本轮不变、target不存在。
