// jarde: presentation of `BGOrder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BGOrder extends java.lang.Object {
    private final Order a;

    private final Order b;

    private int count;

    private boolean fail;

    public BGOrder() {
        // @method <init>()V
        // @declaration a constructor of `BGOrder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = new Order("a");
        this.b = new Order("b");
        this.count = 0;
        this.fail = false;
        return;
    }

    void singleIf() {
        // jarde: not recovered: the recovery run for `singleIf()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method singleIf()V
        // @declaration an instance method of `BGOrder`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31 34 35 37 40 41 42 45 48 49 52 55 58 59 60 63 66 67 70 73 74 75
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }

    void ifElse(boolean arg1) {
        // @method ifElse(Z)V
        // @declaration an instance method of `BGOrder`, member flags 0x0000
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

    void twoIfs(boolean arg1, boolean arg2) {
        // jarde: not recovered: the recovery run for `twoIfs(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twoIfs(ZZ)V
        // @declaration an instance method of `BGOrder`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 29 30 33 34 35 38 39 42 43 44 47 48 49 52 53 56 59 60 63 66 69 70 71 74 77 78 81 84 85 86
        // the shared catch-all finally has no complete bounded try and catch bodies
        jarde_refused_body();
    }

    int count() {
        // @method count()I
        // @declaration an instance method of `BGOrder`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.count;
    }

    void arm(boolean arg1) {
        // @method arm(Z)V
        // @declaration an instance method of `BGOrder`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.fail = arg1;
        return;
    }
}
