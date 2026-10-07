// jarde: presentation of `MixedLocalControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedLocalControls extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static boolean result;

    public MixedLocalControls() {
        // @method <init>()V
        // @declaration a constructor of `MixedLocalControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedLocalControls.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedLocalControls.cValue;
    }

    static int numeric(boolean arg0) {
        // jarde: not recovered: the recovery run for `numeric(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method numeric(Z)I
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 13 16 17 20 21 22 23 24 25
        // the short-circuit chain from BCI 1 through 13 reaches a shared value consumer at BCI 21, but this slice has no SSA proof for that value; the complete region is quoted
    }

    static boolean rewritten(boolean arg0) {
        // jarde: not recovered: the recovery run for `rewritten(Z)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method rewritten(Z)Z
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 13 16 17 20 21 22 23
        // the short-circuit chain from BCI 1 through 13 reaches a shared value consumer at BCI 21, but this slice has no SSA proof for that value; the complete region is quoted
    }

    static boolean duplicated(boolean arg0) {
        // jarde: not recovered: the recovery run for `duplicated(Z)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method duplicated(Z)Z
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 13 16 17 20 21 22 25 26 27
        // the short-circuit chain from BCI 1 through 13 reaches a shared value consumer at BCI 22, but this slice has no SSA proof for that value; the complete region is quoted
    }

    static boolean crossing(boolean arg0) {
        // jarde: not recovered: the recovery run for `crossing(Z)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method crossing(Z)Z
        // @declaration a static method of `MixedLocalControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 16 20 21 25 28
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }
}
