// jarde: presentation of `ChainExtraBoundary` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ChainExtraBoundary extends java.lang.Object {
    static boolean result;

    static boolean rhsValue;

    static int calls;

    public ChainExtraBoundary() {
        // @method <init>()V
        // @declaration a constructor of `ChainExtraBoundary`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `ChainExtraBoundary`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ChainExtraBoundary.calls = ChainExtraBoundary.calls + 1;
        return ChainExtraBoundary.rhsValue;
    }

    static void assign(boolean arg0, boolean arg1, boolean arg2, boolean arg3) {
        // @method assign(ZZZZ)V
        // @declaration a static method of `ChainExtraBoundary`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ChainExtraBoundary.result = (arg0 ? arg1 ? 1 : !arg3 ? rhs() ? 1 : 0 : 1 : !arg2 ? !arg3 ? rhs() ? 1 : 0 : 1 : 1) % 2 != 0;
        return;
    }
}
