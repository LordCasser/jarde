// jarde: presentation of `MixedShortCircuitField` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedShortCircuitField extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static int bCalls;

    static int cCalls;

    static int receiverCalls;

    static MixedShortCircuitField$Box receiver;

    public MixedShortCircuitField() {
        // @method <init>()V
        // @declaration a constructor of `MixedShortCircuitField`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static MixedShortCircuitField$Box target(boolean arg0) {
        // @method target(Z)LMixedShortCircuitField$Box;
        // @declaration a static method of `MixedShortCircuitField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedShortCircuitField.receiverCalls = MixedShortCircuitField.receiverCalls + 1;
        return arg0 ? null : MixedShortCircuitField.receiver;
    }

    static void one(boolean arg0, boolean arg1) {
        // @method one(ZZ)V
        // @declaration a static method of `MixedShortCircuitField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        target(arg1).result = (arg0 ? !b() ? c() ? 1 : 0 : 1 : c() ? 1 : 0) % 2 != 0;
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedShortCircuitField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedShortCircuitField.bCalls = MixedShortCircuitField.bCalls + 1;
        return MixedShortCircuitField.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedShortCircuitField`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedShortCircuitField.cCalls = MixedShortCircuitField.cCalls + 1;
        return MixedShortCircuitField.cValue;
    }
}

