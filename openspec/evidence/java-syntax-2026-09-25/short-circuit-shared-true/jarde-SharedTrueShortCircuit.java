// jarde: presentation of `SharedTrueShortCircuit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class SharedTrueShortCircuit extends java.lang.Object {
    static boolean result;

    static int calls;

    public SharedTrueShortCircuit() {
        // @method <init>()V
        // @declaration a constructor of `SharedTrueShortCircuit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `SharedTrueShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        SharedTrueShortCircuit.calls = SharedTrueShortCircuit.calls + 1;
        return true;
    }

    static void assign(boolean arg0) {
        // jarde: not recovered: the recovery run for `assign(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method assign(Z)V
        // @declaration a static method of `SharedTrueShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 11 14 15 18
        // the short-circuit branches at BCI 1 and 7 reach a shared value consumer at BCI 15, but this slice has no SSA proof for that value; the complete region is quoted
    }
}
