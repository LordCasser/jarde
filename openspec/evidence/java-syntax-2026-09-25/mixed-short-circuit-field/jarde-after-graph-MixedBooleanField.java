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
        MixedBooleanField.result = (arg0 ? !b() ? c() ? 1 : 0 : 1 : c() ? 1 : 0) % 2 != 0;
        return;
    }

    static void orAnd(boolean arg0) {
        // @method orAnd(Z)V
        // @declaration a static method of `MixedBooleanField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanField.result = (!arg0 ? b() ? c() ? 1 : 0 : 0 : c() ? 1 : 0) % 2 != 0;
        return;
    }
}
