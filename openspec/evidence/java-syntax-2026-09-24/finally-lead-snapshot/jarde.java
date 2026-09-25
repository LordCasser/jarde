// jarde: presentation of `FinallyLeadSnapshot` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyLeadSnapshot extends java.lang.Object {
    public static int value;

    public FinallyLeadSnapshot() {
        // @method <init>()V
        // @declaration a constructor of `FinallyLeadSnapshot`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int run() {
        // jarde: not recovered: the recovery run for `run()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run()I
        // @declaration a static method of `FinallyLeadSnapshot`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 16: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 16
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [16]
    }
}
