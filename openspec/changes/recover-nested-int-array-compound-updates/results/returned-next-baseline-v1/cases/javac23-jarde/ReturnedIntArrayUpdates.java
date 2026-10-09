// jarde: presentation of `ReturnedIntArrayUpdates` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ReturnedIntArrayUpdates extends java.lang.Object {
    static int trace;

    public ReturnedIntArrayUpdates() {
        // @method <init>()V
        // @declaration a constructor of `ReturnedIntArrayUpdates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int plain2(int[][] arg0, int arg1, int arg2, int arg3) {
        // jarde: not recovered: the recovery run for `plain2([[IIII)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method plain2([[IIII)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 2 5
        // the dependency chain from BCI 5 to final consumer 8 is not bounded
        // @bytecode 8
        // the instruction at BCI 8 is not part of the provable subset
        // @bytecode 9 2 5
        // the value at BCI 9 comes from an Other at BCI 8, which produces no expression this subset writes
        // @bytecode 10 2 5
        // the value at BCI 10 comes from an Other at BCI 8, which produces no expression this subset writes
    }

    public static int plain3(int[][][] arg0, int arg1, int arg2, int arg3, int arg4) {
        // jarde: not recovered: the recovery run for `plain3([[[IIIII)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method plain3([[[IIIII)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 4 2
        // the array instruction at BCI 4 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 6
        // the instruction at BCI 6 is not part of the provable subset
        // @bytecode 2 4 7
        // the dependency chain from BCI 7 to final consumer 11 is not bounded
        // @bytecode 11
        // the instruction at BCI 11 is not part of the provable subset
        // @bytecode 12 4 2 7
        // the value at BCI 12 comes from an Other at BCI 11, which produces no expression this subset writes
        // @bytecode 13 4 2 7
        // the value at BCI 13 comes from an Other at BCI 11, which produces no expression this subset writes
    }

    public static int scalar(int[] arg0, int arg1, int arg2) {
        // jarde: not recovered: the recovery run for `scalar([III)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method scalar([III)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the dependency chain from BCI 3 to final consumer 6 is not bounded
        // @bytecode 6
        // the instruction at BCI 6 is not part of the provable subset
        // @bytecode 7 3
        // the value at BCI 7 comes from an Other at BCI 6, which produces no expression this subset writes
        // @bytecode 8 3
        // the value at BCI 8 comes from an Other at BCI 6, which produces no expression this subset writes
    }

    public static int traced(int[][] arg0, int arg1, int arg2, int arg3) {
        // @method traced([[IIII)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5 2 0 1
        // the array instruction at BCI 5 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        index(arg2);
        // @bytecode 10
        // the instruction at BCI 10 is not part of the provable subset
        // @bytecode 0 1 2 5 6 11
        // the dependency chain from BCI 11 to final consumer 17 is not bounded
        // @bytecode 12 13
        // the dependency chain from BCI 13 to final consumer 17 is not bounded
        // @bytecode 17
        // the instruction at BCI 17 is not part of the provable subset
        // @bytecode 18 5 2 11 13 0 1 6 12
        // the value at BCI 18 comes from an Other at BCI 17, which produces no expression this subset writes
        // @bytecode 19 5 2 11 13 0 1 6 12
        // the value at BCI 19 comes from an Other at BCI 17, which produces no expression this subset writes
    }

    static int row(int arg0) {
        // @method row(I)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ReturnedIntArrayUpdates.trace = ReturnedIntArrayUpdates.trace * 10 + 1;
        return arg0;
    }

    static int index(int arg0) {
        // @method index(I)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ReturnedIntArrayUpdates.trace = ReturnedIntArrayUpdates.trace * 10 + 2;
        return arg0;
    }

    static int rhs(int arg0) {
        // @method rhs(I)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ReturnedIntArrayUpdates.trace = ReturnedIntArrayUpdates.trace * 10 + 3;
        return arg0;
    }

    public static int swap(int[][] arg0) {
        // @method swap([[I)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0] = new int[]{100};
        return 7;
    }

    public static int replaceRow(int[][] arg0) {
        // jarde: not recovered: the recovery run for `replaceRow([[I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method replaceRow([[I)I
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 2 5
        // the dependency chain from BCI 5 to final consumer 11 is not bounded
        // @bytecode 6 7
        // the dependency chain from BCI 7 to final consumer 11 is not bounded
        // @bytecode 11
        // the instruction at BCI 11 is not part of the provable subset
        // @bytecode 12 2 5 7 0 6
        // the value at BCI 12 comes from an Other at BCI 11, which produces no expression this subset writes
        // @bytecode 13 2 5 7 0 6
        // the value at BCI 13 comes from an Other at BCI 11, which produces no expression this subset writes
    }

    public static void different(int[][] arg0, int arg1) {
        // @method different([[II)V
        // @declaration a static method of `ReturnedIntArrayUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0][0] = arg0[1][0] + arg1;
        return;
    }
}
