// jarde: presentation of `StageRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class StageRunner extends java.lang.Object {
    StageRunner() {
        // @method <init>()V
        // @declaration a constructor of `StageRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void check(boolean arg0, java.lang.String arg1) {
        // @method check(ZLjava/lang/String;)V
        // @declaration a static method of `StageRunner`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0) {
            throw new java.lang.AssertionError((java.lang.Object) arg1);
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StageRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Stage[] local1 = Stage.values();
        if (local1.length == 2) {
        }
        // @bytecode 17
        // the value at BCI 17 is the entry state of stack depth 0, which no instruction produced
        if (local1[0] == Stage.START) {
            if (local1[1] == Stage.FINISH) {
                // @bytecode 45
                // the value at BCI 45 is the entry state of stack depth 0, which no instruction produced
                if (Stage.valueOf("START") == Stage.START) {
                }
            }
        }
        // @bytecode 43
        // block at BCI 43 can be re-entered and belongs to no loop this subset proves
        // @bytecode 64 80 84 85 100 114 110 115 131 146 142 147 175 186 182 187 203 218 214 219
        // 20 live block(s) are reachable only through edges the normal-flow view leaves out: [64, 80, 84, 85, 100, 114, 110, 115, 131, 146, 142, 147, 175, 186, 182, 187, 203, 218, 214, 219]
    }
}
