// jarde: presentation of `BGNegatives` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BGNegatives extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock a;

    private final java.util.concurrent.locks.ReentrantLock b;

    private int count;

    private boolean flag;

    private java.lang.IllegalStateException stored;

    public BGNegatives() {
        // @method <init>()V
        // @declaration a constructor of `BGNegatives`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = new java.util.concurrent.locks.ReentrantLock();
        this.b = new java.util.concurrent.locks.ReentrantLock();
        this.count = 0;
        this.flag = false;
        this.stored = new java.lang.IllegalStateException("stored");
        return;
    }

    void tailThrow() {
        // jarde: not recovered: the recovery run for `tailThrow()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method tailThrow()V
        // @declaration an instance method of `BGNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28
        // BCI 58: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 31 34 35 37 40 41 42 45 48 49 52 55 58 59 60 63 66 67 70 73 74 75 78 79 81 84
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [31, 41, 58]
        jarde_refused_body();
    }

    void tailStoredThrow() {
        // jarde: not recovered: the recovery run for `tailStoredThrow()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method tailStoredThrow()V
        // @declaration an instance method of `BGNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28
        // BCI 58: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 31 34 35 37 40 41 42 45 48 49 52 55 58 59 60 63 66 67 70 73 74 75 76 79
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [31, 41, 58]
        jarde_refused_body();
    }

    void bodyReturn(boolean arg1) {
        // jarde: not recovered: the recovery run for `bodyReturn(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method bodyReturn(Z)V
        // @declaration an instance method of `BGNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25
        // BCI 60: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 28 29 32 35 36 39 42 43 44 47 50 51 54 57 60 61 62 65 68 69 72 75 76 77
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [28, 43, 60]
        jarde_refused_body();
    }

    void switchBody(int arg1) {
        // jarde: not recovered: the recovery run for `switchBody(I)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method switchBody(I)V
        // @declaration an instance method of `BGNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 44 45 46 49 50 51 54 57 58 59 62 63 64 67 68 71 74 75 78 81 84 85 86 89 92 93 96 99 100 101
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }
}
