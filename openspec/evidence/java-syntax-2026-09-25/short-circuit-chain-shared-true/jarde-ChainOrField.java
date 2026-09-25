// jarde: presentation of `ChainOrField` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ChainOrField extends java.lang.Object {
    static boolean result;

    static boolean rhsValue;

    static int calls;

    public ChainOrField() {
        // @method <init>()V
        // @declaration a constructor of `ChainOrField`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `ChainOrField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ChainOrField.calls = ChainOrField.calls + 1;
        return ChainOrField.rhsValue;
    }

    static void assign(boolean arg0, boolean arg1) {
        // @method assign(ZZ)V
        // @declaration a static method of `ChainOrField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0) {
            if (!arg1) {
                if (rhs()) {
                } else {
                    // @bytecode 19
                    // the value at BCI 19 is the entry state of stack depth 0, which no instruction produced
                    return;
                }
            }
        }
        // @bytecode 19
        // block at BCI 19 can be re-entered and belongs to no loop this subset proves
    }
}
