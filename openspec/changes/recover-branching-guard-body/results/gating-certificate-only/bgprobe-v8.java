// jarde: presentation of `BGProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BGProbe extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock a;

    private final java.util.concurrent.locks.ReentrantLock b;

    private int count;

    private boolean flag;

    public BGProbe() {
        // @method <init>()V
        // @declaration a constructor of `BGProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = new java.util.concurrent.locks.ReentrantLock();
        this.b = new java.util.concurrent.locks.ReentrantLock();
        this.count = 0;
        this.flag = false;
        return;
    }

    void twoIfs(boolean arg1, boolean arg2) {
        // jarde: not recovered: the recovery run for `twoIfs(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twoIfs(ZZ)V
        // @declaration an instance method of `BGProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 29 30 33 34 35 38 39 42 43 44 47 48 49 52 53 56 59 60 63 66 69 70 71 74 77 78 81 84 85 86
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }

    void nestedIf(boolean arg1, boolean arg2) {
        // jarde: not recovered: the recovery run for `nestedIf(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedIf(ZZ)V
        // @declaration an instance method of `BGProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 29 32 33 34 37 38 39 42 43 46 49 50 53 56 59 60 61 64 67 68 71 74 75 76
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }

    void bodyLoop() {
        // @method bodyLoop()V
        // @declaration an instance method of `BGProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.a.lock();
        this.b.lock();
        try {
            while (this.flag) {
                this.count += 1;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        return;
    }
}
