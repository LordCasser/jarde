# CF11 debug-metadata control tooling

This private prep contains the existing CF11 collector plus the v6 exception-table observation checker. The collector adds `--source-debug {debug,none}`; it defaults to `debug`, preserving the current compiler behavior. The selected mode applies to every `javac` compile and is recorded both as `compiler_debug_info` in `execution.json` and as `collector_config.source_debug`. The only accepted compiler arguments are exactly `-g` for `debug` and `-g:none` for `none`.

For the no-debug baseline, add `--source-debug none` to the existing collector invocation. For the default baseline, omit the option or pass `--source-debug debug`. The checker validates the receipt/config match and reconstructs every expected compile command with that exact single flag. It retains the v6 pinned-handler exception-table checks and all prior input/tool/raw-stream/source-map/profile/compile/runtime checks.

The collector's source/tool pins still include the collector itself and all original tracked inputs. No collector or checker execution was performed while preparing this directory; run the collector and then the checker against its new receipt after review. This is observation tooling only and makes no product or behavioral acceptance claim.
