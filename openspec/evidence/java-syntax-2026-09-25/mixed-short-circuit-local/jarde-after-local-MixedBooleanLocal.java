// jarde: presentation of `MixedBooleanLocal` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedBooleanLocal extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static boolean result;

    static int bCalls;

    static int cCalls;

    public MixedBooleanLocal() {
        // @method <init>()V
        // @declaration a constructor of `MixedBooleanLocal`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedBooleanLocal`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanLocal.bCalls = MixedBooleanLocal.bCalls + 1;
        return MixedBooleanLocal.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedBooleanLocal`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedBooleanLocal.cCalls = MixedBooleanLocal.cCalls + 1;
        return MixedBooleanLocal.cValue;
    }

    static boolean one(boolean arg0) {
        // @method one(Z)Z
        // @declaration a static method of `MixedBooleanLocal`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 ? !b() ? c() ? 1 : 0 : 1 : c() ? 1 : 0) % 2 != 0;
        MixedBooleanLocal.result = local1;
        return local1;
    }
}
