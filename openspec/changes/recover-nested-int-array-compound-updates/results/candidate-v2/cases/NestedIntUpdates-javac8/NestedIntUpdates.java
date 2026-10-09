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
        // @method plain2([[IIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[arg1][arg2] += arg3;
        return;
    }

    public static void plain3(int[][][] arg0, int arg1, int arg2, int arg3, int arg4) {
        // @method plain3([[[IIIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[arg1][arg2][arg3] += arg4;
        return;
    }

    public static void scalar(int[] arg0, int arg1, int arg2) {
        // @method scalar([III)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[arg1] += arg2;
        return;
    }

    public static void traced(int[][] arg0, int arg1, int arg2, int arg3) {
        // @method traced([[IIII)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[row(arg1)][index(arg2)] += rhs(arg3);
        return;
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
        // @method replaceRow([[I)I
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0][0] += swap(arg0);
        return 7;
    }

    public static void different(int[][] arg0, int arg1) {
        // @method different([[II)V
        // @declaration a static method of `NestedIntUpdates`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0[0][0] = arg0[1][0] + arg1;
        return;
    }
}
