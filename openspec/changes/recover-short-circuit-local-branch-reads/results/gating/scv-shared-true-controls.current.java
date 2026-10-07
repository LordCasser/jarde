// jarde: presentation of `SharedTrueDuplicatePhi` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SharedTrueDuplicatePhi extends java.lang.Object {
    static boolean result;

    static boolean other;

    static int calls;

    public SharedTrueDuplicatePhi() {
        // @method <init>()V
        // @declaration a constructor of `SharedTrueDuplicatePhi`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `SharedTrueDuplicatePhi`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        SharedTrueDuplicatePhi.calls = SharedTrueDuplicatePhi.calls + 1;
        return true;
    }

    static void assign(boolean arg0) {
        // jarde: not recovered: the recovery run for `assign(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method assign(Z)V
        // @declaration a static method of `SharedTrueDuplicatePhi`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 11 14 15 16 19 22
        // the short-circuit chain from BCI 1 through 7 reaches a shared value consumer at BCI 16, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }
}
