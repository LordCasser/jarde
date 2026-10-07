// jarde: presentation of `LK` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LK extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock lock;

    private final java.util.concurrent.locks.Condition notFull;

    private int count;

    public LK() {
        // @method <init>()V
        // @declaration a constructor of `LK`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.lock = new java.util.concurrent.locks.ReentrantLock();
        this.notFull = this.lock.newCondition();
        this.count = 0;
        return;
    }

    void put() throws java.lang.InterruptedException {
        // jarde: not recovered: the recovery run for `put()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method put()V
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 7 15 27 56 66
        // the quoted block at BCI 56 ends in a control-flow exit at BCI 65 (a return, a throw, or a transfer whose destination the presented structure does not own), the presented structure reaches that quote, and the body without it would still compile and silently change what the method does; the whole method is quoted
        jarde_refused_body();
    }

    int take() throws java.lang.InterruptedException {
        // @method take()I
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.lock.lock();
        // @bytecode 7 8 11
        // block at BCI 7 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        // @bytecode 14 15 18 23 26 27 28 31 32 33 36 37 40 45 46 49 50 51 54 57 58 59 60 61 64 67 68
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [14, 26, 59]
    }

    boolean tryLockQuick() {
        // @method tryLockQuick()Z
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        if (this.lock.tryLock()) {
            // @bytecode 10 11 12 15 17 18 21 22 23 24 27 30 31
            // BCI 32: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        } else {
            return false;
        }
        // @bytecode 32 33 34 37 40 41
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [32]
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `LK`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        LK local1 = new LK();
        local1.put();
        local1.put();
        java.lang.System.out.println("" + local1.take() + "/" + local1.take() + "/" + local1.tryLockQuick());
        return;
    }
}
