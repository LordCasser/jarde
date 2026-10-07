# Task 1.1 gating + 1.2 baseline: the reorder acceptance alone flips DB's capture form

Date: 2026-10-07 (UTC). Binary: this worktree, `cargo build --locked -p jarde-cli`.

## 1.2 baseline, before the change (patrol jar)

```
$ ./target/debug/jarde-cli class-source --input openspec/evidence/java-syntax-2026-10-05/double-brace-patrol/fixture/db.jar --class 'DB$2' --format text --output /tmp/base-DB2.java
class DB$2 extends java.util.ArrayList {
    final java.lang.String val$s;

    DB$2(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `DB$2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$s = arg1;
        super();
        this.add((java.lang.Object) this.val$s);
        return;
    }
}

$ javac --release 8 -g:none -d out DB.java 'DB$1.java' 'DB$2.java'      # ambient javac 23
DB$2.java:12: 错误: 灵活构造器 是预览功能，默认情况下禁用。
        this.val$s = arg1;
        ^
  （请使用 --enable-preview 以启用 灵活构造器）
1 个错误                                                               # exit 1 — the patrol's finding, reproduced
```

`DB$1` (no capture) renders `super();` first and `DB` renders `return new DB$2(arg0);` — both
healthy at baseline (`/tmp/base-DB1.java`, `/tmp/base-DB.java`).

## The gating experiment: the reorder acceptance alone, then the same three renders

```
$ ./target/debug/jarde-cli class-source --input .../db.jar --class 'DB$2' --format text --output /tmp/fix-DB2.java
class DB$2 extends java.util.ArrayList {
    final java.lang.String val$s;

    DB$2(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `DB$2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.val$s = arg1;
        this.add((java.lang.Object) this.val$s);
        return;
    }
}

$ ./target/debug/jarde-cli class-source --input .../db.jar --class 'DB$1' --format text --output /tmp/fix-DB1.java
$ diff /tmp/base-DB1.java /tmp/fix-DB1.java && echo "DB\$1 byte-identical"
DB$1 byte-identical
$ ./target/debug/jarde-cli class-source --input .../db.jar --class DB --format text --output /tmp/fix-DB.java
$ diff /tmp/base-DB.java /tmp/fix-DB.java && echo "host byte-identical"
host byte-identical
```

## The recompiled family, both fixture legs and both javacs

```
$ javac --release 8 -g:none -d out23 DB.java DB\$1.java DB\$2.java   # ambient javac 23, exit 0
$ <jdk8>/bin/javac -g:none -d out8 DB.java DB\$1.java DB\$2.java     # real javac 8, exit 0
$ java -Xverify:all -cp out23 DB
2/z
$ <jdk8>/bin/java -Xverify:all -cp out8 DB
2/z
```

Run over three class sets: the patrol jar (javac 8 bytes), the frozen javac-23 leg
(`tests/fixtures/proved-java-structure/double-brace-capture/v23/`) and the frozen javac-8 leg
(`.../v8/`); all six compile+run pairs print `2/z` — the same output the original classes print
(patrol evidence `results/o-DB_1.txt`, `o-DB_2.txt`; the fixture's `freeze.py` re-runs both legs
under `java -Xverify:all` and requires it).

## The order-sensitive standing controls, run at this point

```
$ cargo test --test ctor_reorder_dispatch_guard --all-features --locked
test result: ok. 2 passed; 0 failed; 0 ignored
$ cargo test --test fixture_behavior_guards --all-features --locked
test result: ok. 7 passed; 0 failed; 6 ignored
$ cargo test --test fixture_behavior_guards --all-features --locked -- --ignored
test result: ok. 6 passed; 0 failed; 0 ignored
```

The dispatch fixture's `visibleDuringSuper` behavior is untouched: its child keeps
`this.val$captured = arg1;` before `super();`, and the recompiled-run comparison in the guard
still has nothing to compare (the text stays refused by javac).
