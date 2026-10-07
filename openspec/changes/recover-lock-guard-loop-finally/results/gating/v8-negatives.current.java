// jarde: presentation of `LockGuardNegatives` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LockGuardNegatives extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock lock;

    private final java.util.concurrent.locks.ReentrantLock other;

    private final java.io.Closeable resource;

    private final java.io.BufferedReader reader;

    private int count;

    public LockGuardNegatives() {
        // @method <init>()V
        // @declaration a constructor of `LockGuardNegatives`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.lock = new java.util.concurrent.locks.ReentrantLock();
        this.other = new java.util.concurrent.locks.ReentrantLock();
        this.resource = new java.io.StringWriter();
        this.reader = null;
        this.count = 0;
        return;
    }

    void lockMismatch() {
        // jarde: not recovered: the recovery run for `lockMismatch()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method lockMismatch()V
        // @declaration an instance method of `LockGuardNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 9 12 13 14 17 18 21 24
        // BCI 27: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 27 28 29 32 35 36 37
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [37, 27]
        jarde_refused_body();
    }

    int guardedRelease() throws java.io.IOException {
        // jarde: not recovered: the recovery run for `guardedRelease()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method guardedRelease()I
        // @declaration an instance method of `LockGuardNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 22 28 39
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    void localLockRewritten() {
        // jarde: not recovered: the recovery run for `localLockRewritten()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method localLockRewritten()V
        // @declaration an instance method of `LockGuardNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 31 38
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
        jarde_refused_body();
    }

    void throwingRelease() throws java.io.IOException {
        // jarde: not recovered: the recovery run for `throwingRelease()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method throwingRelease()V
        // @declaration an instance method of `LockGuardNegatives`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 9 12 13 14 17 18 21 26
        // block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        // @bytecode 29 30 31 34 39 40 41
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [41, 29]
        jarde_refused_body();
    }
}
