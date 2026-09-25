// jarde: presentation of `Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Probe extends java.lang.Object {
    public Probe() {
        // @method <init>()V
        // @declaration a constructor of `Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int sumAndReturnIndex(int[] arg0) {
        // jarde: not recovered: the recovery run for `sumAndReturnIndex([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sumAndReturnIndex([I)I
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 22
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int mismatchedArrays(int[] arg0, int[] arg1) {
        // jarde: not recovered: the recovery run for `mismatchedArrays([I[I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method mismatchedArrays([I[I)I
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 22
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 35: the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect
        // @bytecode 119 87
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [119, 87]
    }
}
