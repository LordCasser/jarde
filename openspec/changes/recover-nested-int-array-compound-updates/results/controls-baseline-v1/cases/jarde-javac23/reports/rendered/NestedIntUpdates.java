// jarde: presentation of `NestedIntUpdates` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedIntUpdates extends java.lang.Object {
    static int trace;

    public NestedIntUpdates() {
        // @method <init>()V
        // @declaration a constructor of `NestedIntUpdates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void plain2(int[][] arg0, int arg1, int arg2, int arg3) {
        // jarde: not recovered: the recovery run for `plain2([[IIII)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method plain2([[IIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 2 5 0 1 3 4 6 7 8 9
        // the dependency chain from BCI 5 to final consumer 8 is not bounded
        // @bytecode 8 2 5
        // the value at BCI 8 comes from an Other at BCI 4, which produces no expression this subset writes
        jarde_refused_body();
    }

    public static void plain3(int[][][] arg0, int arg1, int arg2, int arg3, int arg4) {
        // jarde: not recovered: the recovery run for `plain3([[[IIIII)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method plain3([[[IIIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 4 2
        // the array instruction at BCI 4 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 6
        // the instruction at BCI 6 is not part of the provable subset
        // @bytecode 2 4 7 0 1 3 5 6 8 10 11 12
        // the dependency chain from BCI 7 to final consumer 11 is not bounded
        // @bytecode 11 4 2 7
        // the value at BCI 11 comes from an Other at BCI 6, which produces no expression this subset writes
        jarde_refused_body();
    }

    public static void scalar(int[] arg0, int arg1, int arg2) {
        // @method scalar([III)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[arg1] += arg2;
        return;
    }

    public static void traced(int[][] arg0, int arg1, int arg2, int arg3) {
        // jarde: not recovered: the recovery run for `traced([[IIII)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method traced([[IIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5 2 0 1
        // the array instruction at BCI 5 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 10
        // the instruction at BCI 10 is not part of the provable subset
        // @bytecode 0 1 2 5 6 11 7 10 12 13 16 17 18
        // the dependency chain from BCI 11 to final consumer 17 is not bounded
        // @bytecode 12 13
        // the dependency chain from BCI 13 to final consumer 17 is not bounded
        // @bytecode 17 5 2 11 13 0 1 6 12
        // the value at BCI 17 comes from an Other at BCI 10, which produces no expression this subset writes
        jarde_refused_body();
    }

    static int row(int arg0) {
        // @method row(I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedIntUpdates.trace = NestedIntUpdates.trace * 10 + 1;
        return arg0;
    }

    static int index(int arg0) {
        // @method index(I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedIntUpdates.trace = NestedIntUpdates.trace * 10 + 2;
        return arg0;
    }

    static int rhs(int arg0) {
        // @method rhs(I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedIntUpdates.trace = NestedIntUpdates.trace * 10 + 3;
        return arg0;
    }

    public static int swap(int[][] arg0) {
        // @method swap([[I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0] = new int[]{100};
        return 7;
    }

    public static int replaceRow(int[][] arg0) {
        // jarde: not recovered: the recovery run for `replaceRow([[I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method replaceRow([[I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 2 5 0 1 3 4 6 7 10 11 12 14
        // the dependency chain from BCI 5 to final consumer 11 is not bounded
        // @bytecode 6 7
        // the dependency chain from BCI 7 to final consumer 11 is not bounded
        // @bytecode 11 2 5 7 0 6
        // the value at BCI 11 comes from an Other at BCI 4, which produces no expression this subset writes
    }

    public static void different(int[][] arg0, int arg1) {
        // @method different([[II)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0][0] = arg0[1][0] + arg1;
        return;
    }
}
