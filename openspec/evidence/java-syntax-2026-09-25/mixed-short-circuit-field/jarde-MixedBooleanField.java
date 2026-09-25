// jarde: presentation of `MixedBooleanField` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedBooleanField extends java.lang.Object {
    static boolean result;

    static boolean bValue;

    static boolean cValue;

    static int bCalls;

    static int cCalls;

    public MixedBooleanField() {
        // @method <init>()V
        // @declaration a constructor of `MixedBooleanField`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedBooleanField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanField.bCalls = MixedBooleanField.bCalls + 1;
        return MixedBooleanField.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedBooleanField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanField.cCalls = MixedBooleanField.cCalls + 1;
        return MixedBooleanField.cValue;
    }

    static void andOr(boolean arg0) {
        // @method andOr(Z)V
        // @declaration a static method of `MixedBooleanField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0) {
            if (!b()) {
            } else {
                // @bytecode 21
                // the value at BCI 21 is the entry state of stack depth 0, which no instruction produced
                return;
            }
        }
        if (c()) {
            // @bytecode 16
            // block at BCI 16 can be re-entered and belongs to no loop this subset proves
        }
        // @bytecode 21
        // block at BCI 21 can be re-entered and belongs to no loop this subset proves
    }

    static void orAnd(boolean arg0) {
        // @method orAnd(Z)V
        // @declaration a static method of `MixedBooleanField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0) {
            if (b()) {
            } else {
                // @bytecode 21
                // the value at BCI 21 is the entry state of stack depth 0, which no instruction produced
                return;
            }
        }
        if (c()) {
        } else {
            // @bytecode 20
            // block at BCI 20 can be re-entered and belongs to no loop this subset proves
        }
        // @bytecode 21
        // block at BCI 21 can be re-entered and belongs to no loop this subset proves
    }
}
