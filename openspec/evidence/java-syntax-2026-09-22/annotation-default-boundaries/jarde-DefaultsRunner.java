// jarde: presentation of `DefaultsRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class DefaultsRunner extends java.lang.Object {
    DefaultsRunner() {
        // @method <init>()V
        // @declaration a constructor of `DefaultsRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void check(boolean arg0, java.lang.String arg1) {
        // @method check(ZLjava/lang/String;)V
        // @declaration a static method of `DefaultsRunner`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0) {
            throw new java.lang.AssertionError((java.lang.Object) arg1);
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `DefaultsRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.reflect.Method local1;
        local1 = Defaults.class.getMethod("nested", new java.lang.Class[0]);
        if (((Inner) local1.getDefaultValue()).count() == 9) {
        }
        // @bytecode 36
        // the value at BCI 36 is the entry state of stack depth 0, which no instruction produced
        local1 = Defaults.class.getMethod("nestedArray", new java.lang.Class[0]);
        Inner[] local2 = (Inner[]) local1.getDefaultValue();
        if (local2.length == 2) {
            if (local2[0].count() == 2) {
                if (local2[1].count() == 4) {
                    // @bytecode 96
                    // the value at BCI 96 is the entry state of stack depth 0, which no instruction produced
                    check(Defaults.class.getMethod("klass", new java.lang.Class[0]).getDefaultValue().equals((java.lang.Object) java.lang.String[].class), "class literal");
                    if (Defaults.class.getMethod("tone", new java.lang.Class[0]).getDefaultValue() == Tone.LOUD) {
                    }
                }
            }
        }
        // @bytecode 94
        // block at BCI 94 can be re-entered and belongs to no loop this subset proves
        // @bytecode 148 174 178 179 248 252 253 288 292 293
        // 10 live block(s) are reachable only through edges the normal-flow view leaves out: [148, 174, 178, 179, 248, 252, 253, 288, 292, 293]
    }
}
