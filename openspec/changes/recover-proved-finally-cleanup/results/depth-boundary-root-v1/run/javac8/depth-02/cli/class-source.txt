// jarde: presentation of `DeepFinally` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DeepFinally extends java.lang.Object {
    static int trace;

    public DeepFinally() {
        // @method <init>()V
        // @declaration a constructor of `DeepFinally`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static void cleanup() {
        // @method cleanup()V
        // @declaration a static method of `DeepFinally`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        DeepFinally.trace = DeepFinally.trace + 1;
        return;
    }

    static int run(int arg0) {
        // jarde: not recovered: the recovery run for `run(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run(I)I
        // @declaration a static method of `DeepFinally`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5
        // BCI 26: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 0 1 2
        // the loop whose header is the block at BCI 0 has a test, an exit or a latch this subset does not prove
        // @bytecode 8 9 10 13 16 19 20 21 24 25 26 27 30 31
        // 4 live block(s) are reachable only through edges the normal-flow view leaves out: [19, 26, 8, 13]
    }
}
