// jarde: presentation of `ChainOrFieldDuplicatePhi` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ChainOrFieldDuplicatePhi extends java.lang.Object {
    static boolean result;

    static boolean mirror;

    static boolean rhsValue;

    static int calls;

    public ChainOrFieldDuplicatePhi() {
        // @method <init>()V
        // @declaration a constructor of `ChainOrFieldDuplicatePhi`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `ChainOrFieldDuplicatePhi`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ChainOrFieldDuplicatePhi.calls = ChainOrFieldDuplicatePhi.calls + 1;
        return ChainOrFieldDuplicatePhi.rhsValue;
    }

    static void assign(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `assign(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method assign(ZZ)V
        // @declaration a static method of `ChainOrFieldDuplicatePhi`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 11 14 15 18 19 20 23 26
        // the short-circuit chain from BCI 1 through 11 reaches a shared value consumer at BCI 20, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }
}
