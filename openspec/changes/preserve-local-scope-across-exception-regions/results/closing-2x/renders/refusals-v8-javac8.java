// jarde: presentation of `ScopeRefusals` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ScopeRefusals extends java.lang.Object {
    private ScopeRefusals() {
        // @method <init>()V
        // @declaration a constructor of `ScopeRefusals`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int savedAcrossFinally(int arg0) {
        // jarde: not recovered: the recovery run for `savedAcrossFinally(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method savedAcrossFinally(I)I
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 17 22
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int handlerComputed(int arg0) {
        // jarde: not recovered: the recovery run for `handlerComputed(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method handlerComputed(I)I
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 16 20 22 23
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int nestedHandler(int arg0) {
        // jarde: not recovered: the recovery run for `nestedHandler(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedHandler(I)I
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 13 16 20
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    static int siblingKept(int arg0) {
        // @method siblingKept(I)I
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = arg0;
        try {
            local1 = local1 + 1;
        } catch (java.lang.RuntimeException local2) {
            local1 = -1;
        }
        return local1;
    }

    static int quotedSliceKept(int arg0) {
        // @method quotedSliceKept(I)I
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = arg0;
        if (arg0 > 0) {
            int local2 = arg0;
            local1 = local1 + local2;
            local2 = local2 - 1;
        }
        return local1;
        // @bytecode 19 20 21 22 23 24 25
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [19]
    }

    static void log(int arg0) {
        // @method log(I)V
        // @declaration a static method of `ScopeRefusals`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
