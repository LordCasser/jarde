// jarde: presentation of `NestedIntBoundaries` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NestedIntBoundaries extends java.lang.Object {
    public NestedIntBoundaries() {
        // @method <init>()V
        // @declaration a constructor of `NestedIntBoundaries`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int returned(int[][] arg0, int arg1, int arg2, int arg3) {
        // jarde: not recovered: the recovery run for `returned([[IIII)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method returned([[IIII)I
        // @declaration a static method of `NestedIntBoundaries`, member flags 0x0009
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

    public static void merged(boolean arg0, int[][] arg1, int[][] arg2, int arg3) {
        // jarde: not recovered: the recovery run for `merged(Z[[I[[II)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method merged(Z[[I[[II)V
        // @declaration a static method of `NestedIntBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 18
        // the instruction at BCI 18 is not part of the provable subset
        // @bytecode 19 0 1 4 5 6 7 10 11 12 13 15 17 18 20 21 22 23
        // the dependency chain from BCI 19 to final consumer 22 is not bounded
        // @bytecode 22 19
        // the value at BCI 22 comes from an Other at BCI 18, which produces no expression this subset writes
        jarde_refused_body();
    }
}
