// jarde: presentation of `MixedArgumentControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedArgumentControls extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static final MixedArgumentControls INSTANCE;

    public MixedArgumentControls() {
        // @method <init>()V
        // @declaration a constructor of `MixedArgumentControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedArgumentControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedArgumentControls.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedArgumentControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedArgumentControls.cValue;
    }

    static void two(boolean arg0, int arg1) {
        // @method two(ZI)V
        // @declaration a static method of `MixedArgumentControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    void take(boolean arg1) {
        // @method take(Z)V
        // @declaration an instance method of `MixedArgumentControls`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static void many(boolean arg0) {
        // jarde: not recovered: the recovery run for `many(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method many(Z)V
        // @declaration a static method of `MixedArgumentControls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 13 16 17 20 21 23 26
        // the short-circuit chain from BCI 1 through 13 reaches a shared value consumer at BCI 23, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }

    public static void instance(boolean arg0) {
        // jarde: not recovered: the recovery run for `instance(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method instance(Z)V
        // @declaration a static method of `MixedArgumentControls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 10 13 16 19 20 23 24 27
        // the short-circuit chain from BCI 4 through 16 reaches a shared value consumer at BCI 24, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `MixedArgumentControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        INSTANCE = new MixedArgumentControls();
    }
}
