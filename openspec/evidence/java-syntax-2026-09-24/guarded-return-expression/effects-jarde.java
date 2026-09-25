// jarde: presentation of `GuardReturnEffects` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class GuardReturnEffects extends java.lang.Object {
    private static final java.lang.Object LOCK;

    static int calls;

    int n;

    public GuardReturnEffects() {
        // @method <init>()V
        // @declaration a constructor of `GuardReturnEffects`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.RuntimeException afterRead(GuardReturnEffects arg0, boolean arg1) {
        // @method afterRead(LGuardReturnEffects;Z)Ljava/lang/RuntimeException;
        // @declaration a static method of `GuardReturnEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        GuardReturnEffects.calls = GuardReturnEffects.calls + 1;
        arg0.n += 10;
        if (arg1) {
            return new java.lang.IllegalStateException("after-read");
        } else {
            return null;
        }
    }

    int effect(boolean arg1) {
        // jarde: not recovered: the recovery run for `effect(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method effect(Z)I
        // @declaration an instance method of `GuardReturnEffects`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 21 24 28
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    int nested(boolean arg1) {
        // jarde: not recovered: the recovery run for `nested(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nested(Z)I
        // @declaration an instance method of `GuardReturnEffects`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 28 31 38 45
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `GuardReturnEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        LOCK = new java.lang.Object();
    }
}
