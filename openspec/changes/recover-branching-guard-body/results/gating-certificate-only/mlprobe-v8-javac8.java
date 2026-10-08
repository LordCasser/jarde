// jarde: presentation of `MLProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MLProbe extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock a;

    private final java.util.concurrent.locks.ReentrantLock b;

    private final java.util.concurrent.locks.ReentrantLock c;

    private int count;

    public MLProbe() {
        // @method <init>()V
        // @declaration a constructor of `MLProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = new java.util.concurrent.locks.ReentrantLock();
        this.b = new java.util.concurrent.locks.ReentrantLock();
        this.c = new java.util.concurrent.locks.ReentrantLock();
        this.count = 0;
        return;
    }

    void nestedTry() {
        // jarde: not recovered: the recovery run for `nestedTry()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedTry()V
        // @declaration an instance method of `MLProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31
        // BCI 54: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 34 35 36 39 42 43 44 45 48 51 54 55 56 59 62 63 64
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [44, 34, 54]
        jarde_refused_body();
    }

    void threeLocks() {
        // jarde: not recovered: the recovery run for `threeLocks()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method threeLocks()V
        // @declaration an instance method of `MLProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 18 21 22 23 26 27 28 31 32 35 38 39 42 45 46 49 52
        // BCI 55: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 55 56 57 60 63 64 67 70 71 74 77 78 79
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [79, 55]
        jarde_refused_body();
    }

    void nestedLocksBranching(boolean arg1) {
        // jarde: not recovered: the recovery run for `nestedLocksBranching(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedLocksBranching(Z)V
        // @declaration an instance method of `MLProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31 32 34 37 38 39 42 45 46 49 52 55 56 57 60 63 64 67 70 71 72
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }
}
