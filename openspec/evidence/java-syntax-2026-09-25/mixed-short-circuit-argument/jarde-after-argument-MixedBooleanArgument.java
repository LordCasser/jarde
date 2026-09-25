// jarde: presentation of `MixedBooleanArgument` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedBooleanArgument extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static boolean result;

    static int bCalls;

    static int cCalls;

    static int sinkCalls;

    public MixedBooleanArgument() {
        // @method <init>()V
        // @declaration a constructor of `MixedBooleanArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedBooleanArgument`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanArgument.bCalls = MixedBooleanArgument.bCalls + 1;
        return MixedBooleanArgument.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedBooleanArgument`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanArgument.cCalls = MixedBooleanArgument.cCalls + 1;
        return MixedBooleanArgument.cValue;
    }

    static void sink(boolean arg0) {
        // @method sink(Z)V
        // @declaration a static method of `MixedBooleanArgument`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanArgument.sinkCalls = MixedBooleanArgument.sinkCalls + 1;
        MixedBooleanArgument.result = arg0;
        return;
    }

    public static void call(boolean arg0) {
        // @method call(Z)V
        // @declaration a static method of `MixedBooleanArgument`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        sink((arg0 ? !b() ? c() ? 1 : 0 : 1 : c() ? 1 : 0) % 2 != 0);
        return;
    }
}
