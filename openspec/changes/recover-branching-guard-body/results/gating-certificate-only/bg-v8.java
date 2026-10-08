// jarde: presentation of `BG` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BG extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock a;

    private final java.util.concurrent.locks.ReentrantLock b;

    private int count;

    public BG() {
        // @method <init>()V
        // @declaration a constructor of `BG`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = new java.util.concurrent.locks.ReentrantLock();
        this.b = new java.util.concurrent.locks.ReentrantLock();
        this.count = 0;
        return;
    }

    void singleIf(boolean arg1) {
        // jarde: not recovered: the recovery run for `singleIf(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method singleIf(Z)V
        // @declaration an instance method of `BG`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31 32 34 37 38 39 42 45 46 49 52 55 56 57 60 63 64 67 70 71 72
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }

    void ifElse(boolean arg1) {
        // @method ifElse(Z)V
        // @declaration an instance method of `BG`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.a.lock();
        this.b.lock();
        try {
            if (arg1) {
                this.count += 1;
            } else {
                this.count -= 1;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        return;
    }
}
