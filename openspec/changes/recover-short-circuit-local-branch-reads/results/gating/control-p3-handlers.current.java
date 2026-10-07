// jarde: presentation of `Guarded` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Guarded extends java.lang.Object {
    static final java.lang.Object LOCK;

    static boolean FLAG;

    public Guarded() {
        // @method <init>()V
        // @declaration a constructor of `Guarded`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Res open(java.lang.String arg0) {
        // @method open(Ljava/lang/String;)LRes;
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("open " + arg0);
        return new Res(arg0, false);
    }

    static Res openFailing(java.lang.String arg0) {
        // @method openFailing(Ljava/lang/String;)LRes;
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("open " + arg0);
        return new Res(arg0, true);
    }

    static Res fail(java.lang.String arg0) {
        // @method fail(Ljava/lang/String;)LRes;
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("fail " + arg0);
        throw new java.lang.IllegalStateException("open-" + arg0);
    }

    static void body() {
        // @method body()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("body");
        return;
    }

    static void tail() {
        // @method tail()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("tail");
        return;
    }

    static void boom() {
        // @method boom()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.lang.IllegalStateException("boom");
    }

    static void one() {
        // @method one()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = open("r")) {
            body();
        }
        return;
    }

    static void two() {
        // @method two()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = open("r"); Res local1 = open("s")) {
            body();
        }
        return;
    }

    static void three() {
        // @method three()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = open("r"); Res local1 = open("s"); Res local2 = open("t")) {
            body();
        }
        return;
    }

    static void suppressed() {
        // @method suppressed()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = openFailing("r")) {
            boom();
        }
        return;
    }

    static void secondInitFails() {
        // @method secondInitFails()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = open("r"); Res local1 = fail("s")) {
            body();
        }
        return;
    }

    static void sync() {
        // @method sync()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (Guarded.LOCK) {
            body();
        }
        return;
    }

    static void syncBody() {
        // @method syncBody()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (Guarded.class) {
            body();
            tail();
        }
        return;
    }

    static void syncThrows() {
        // @method syncThrows()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (Guarded.LOCK) {
            boom();
        }
        return;
    }

    static void withCatch() {
        // @method withCatch()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try (Res local0 = open("r")) {
            body();
        } catch (java.lang.RuntimeException local0) {
            tail();
        }
        return;
    }

    static void branching() {
        // jarde: not recovered: the recovery run for `branching()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method branching()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 12 15 19 26 31 38 44 46
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
        jarde_refused_body();
    }

    static void fin() {
        // jarde: not recovered: the recovery run for `fin()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method fin()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 6
        // BCI 9: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 9 10 13 14 15
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [15, 9]
        jarde_refused_body();
    }

    static void catchFinally() {
        // jarde: not recovered: the recovery run for `catchFinally()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method catchFinally()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 6
        // BCI 19: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 9 10 13 16 19 20 23 24 25
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [25, 9, 19]
        jarde_refused_body();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Guarded`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        one();
        two();
        three();
        sync();
        syncBody();
        syncThrowsCatching();
        secondInitFailsCatching();
        suppressedCatching();
        branching();
        withCatch();
        fin();
        catchFinally();
        return;
    }

    static void syncThrowsCatching() {
        // @method syncThrowsCatching()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            syncThrows();
        } catch (java.lang.RuntimeException local0) {
            java.lang.System.out.println("caught " + local0.getMessage());
        }
        return;
    }

    static void secondInitFailsCatching() {
        // @method secondInitFailsCatching()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            secondInitFails();
        } catch (java.lang.RuntimeException local0) {
            java.lang.System.out.println("caught " + local0.getMessage());
        }
        return;
    }

    static void suppressedCatching() {
        // jarde: not recovered: the recovery run for `suppressedCatching()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method suppressedCatching()V
        // @declaration a static method of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 6 45 50 90
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
        jarde_refused_body();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Guarded`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        LOCK = new java.lang.Object();
        Guarded.FLAG = true;
    }
}
