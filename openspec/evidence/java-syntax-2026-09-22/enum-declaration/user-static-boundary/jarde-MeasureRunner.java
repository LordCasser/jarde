// jarde: presentation of `MeasureRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class MeasureRunner extends java.lang.Object {
    MeasureRunner() {
        // @method <init>()V
        // @declaration a constructor of `MeasureRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void check(boolean arg0, java.lang.String arg1) {
        // @method check(ZLjava/lang/String;)V
        // @declaration a static method of `MeasureRunner`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0) {
            throw new java.lang.AssertionError((java.lang.Object) arg1);
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MeasureRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Measure[] local1;
        local1 = Measure.values();
        if (local1.length == 2) {
        }
        // @bytecode 17
        // the value at BCI 17 is the entry state of stack depth 0, which no instruction produced
        if (local1[0] == Measure.LOW) {
            if (local1[1] == Measure.HIGH) {
                // @bytecode 45
                // the value at BCI 45 is the entry state of stack depth 0, which no instruction produced
                if (Measure.valueOf("LOW") == Measure.LOW) {
                }
            }
        }
        // @bytecode 43
        // block at BCI 43 can be re-entered and belongs to no loop this subset proves
        // @bytecode 64 79 93 89 94 107 119 115 120
        // 9 live block(s) are reachable only through edges the normal-flow view leaves out: [64, 79, 93, 89, 94, 107, 119, 115, 120]
    }
}
