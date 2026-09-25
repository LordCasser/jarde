// jarde: presentation of `ShiftSlice` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ShiftSlice extends java.lang.Object {
    public ShiftSlice() {
        // @method <init>()V
        // @declaration a constructor of `ShiftSlice`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int intLeft(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `intLeft(II)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method intLeft(II)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static int intRight(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `intRight(II)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method intRight(II)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static int intUnsigned(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `intUnsigned(II)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method intUnsigned(II)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static long longLeft(long arg0, int arg2) {
        // jarde: not recovered: the recovery run for `longLeft(JI)J` produced no statement (explanation only); the artifact's own comment lines are below
        // @method longLeft(JI)J
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static long longRight(long arg0, int arg2) {
        // jarde: not recovered: the recovery run for `longRight(JI)J` produced no statement (explanation only); the artifact's own comment lines are below
        // @method longRight(JI)J
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static long longUnsigned(long arg0, int arg2) {
        // jarde: not recovered: the recovery run for `longUnsigned(JI)J` produced no statement (explanation only); the artifact's own comment lines are below
        // @method longUnsigned(JI)J
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static int shortLeft(short arg0, int arg1) {
        // jarde: not recovered: the recovery run for `shortLeft(SI)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method shortLeft(SI)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static int charUnsigned(char arg0, int arg1) {
        // jarde: not recovered: the recovery run for `charUnsigned(CI)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method charUnsigned(CI)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public static int nested(int arg0, int arg1, int arg2) {
        // jarde: not recovered: the recovery run for `nested(III)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nested(III)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 5
        // the value at BCI 5 comes from an Other at BCI 4, which produces no expression this subset writes
    }

    public static int callTarget(int arg0, int arg1) {
        // @method callTarget(II)I
        // @declaration a static method of `ShiftSlice`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ShiftSliceHelper.value(arg0);
        ShiftSliceHelper.distance(arg1);
        // @bytecode 8
        // the instruction at BCI 8 is not part of the provable subset
        // @bytecode 9
        // the value at BCI 9 comes from an Other at BCI 8, which produces no expression this subset writes
    }
}
