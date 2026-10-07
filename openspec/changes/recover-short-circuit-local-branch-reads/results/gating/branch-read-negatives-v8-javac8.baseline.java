// jarde: presentation of `BranchReadNegatives` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BranchReadNegatives extends java.lang.Object {
    private BranchReadNegatives() {
        // @method <init>()V
        // @declaration a constructor of `BranchReadNegatives`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int loopCondition(int x) {
        // @method loopCondition(I)I
        // @declaration a static method of `BranchReadNegatives`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int b;
        int n;
        // @bytecode 0 3 4 7 8 11 12 15 16 17 18
        // the short-circuit chain from BCI 4 through 8 reaches a shared value consumer at BCI 16, but this slice has no SSA proof for that value; the complete region is quoted
        while (b != 0) {
            n = n + 1;
            if (n > 2) {
                break;
            }
        }
        return n;
    }

    static int crossCatch(int x) {
        // jarde: not recovered: the recovery run for `crossCatch(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method crossCatch(I)I
        // @declaration a static method of `BranchReadNegatives`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 10 11 14 15 18 19 20 21 24 25 28 29
        // canonical block at BCI 18 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BranchReadNegatives`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(loopCondition(0));
        java.lang.System.out.println(loopCondition(-2));
        java.lang.System.out.println(crossCatch(0));
        return;
    }
}
