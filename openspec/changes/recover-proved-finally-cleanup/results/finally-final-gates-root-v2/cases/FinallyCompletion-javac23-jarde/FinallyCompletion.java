// jarde: presentation of `FinallyCompletion` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyCompletion extends java.lang.Object {
    public static int trace;

    public static final java.lang.RuntimeException TRY_FAILURE;

    public static final java.lang.RuntimeException FINALLY_FAILURE;

    public FinallyCompletion() {
        // @method <init>()V
        // @declaration a constructor of `FinallyCompletion`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `FinallyCompletion`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        FinallyCompletion.trace = FinallyCompletion.trace * 10 + arg0;
        return arg0;
    }

    public static int normalReturn() {
        // @method normalReturn()I
        // @declaration a static method of `FinallyCompletion`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            int local0 = mark(1);
            return local0;
        } finally {
            mark(2);
        }
    }

    public static int finallyReturns() {
        // jarde: not recovered: the recovery run for `finallyReturns()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method finallyReturns()I
        // @declaration a static method of `FinallyCompletion`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 6 9
        // block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        // @bytecode 10 11 12 15
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [10]
    }

    public static int finallyThrows() {
        // jarde: not recovered: the recovery run for `finallyThrows()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method finallyThrows()I
        // @declaration a static method of `FinallyCompletion`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 7 10 11 14
        // block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        // @bytecode 15 16 18 21 22 25
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [15]
    }

    public static int tryThrowsFinallyRuns() {
        // jarde: not recovered: the recovery run for `tryThrowsFinallyRuns()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method tryThrowsFinallyRuns()I
        // @declaration a static method of `FinallyCompletion`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 5 6 9
        // BCI 10: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 10 11 13 16 17 18
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [10]
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `FinallyCompletion`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        TRY_FAILURE = new java.lang.IllegalArgumentException("try");
        FINALLY_FAILURE = new java.lang.IllegalStateException("finally");
    }
}
