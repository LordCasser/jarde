// jarde: presentation of `ScopePlanCrossing` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ScopePlanCrossing extends java.lang.Object {
    private ScopePlanCrossing() {
        // @method <init>()V
        // @declaration a constructor of `ScopePlanCrossing`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int flatFinally(int arg0) {
        // jarde: not recovered: the recovery run for `flatFinally(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method flatFinally(I)I
        // @declaration a static method of `ScopePlanCrossing`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 15 28
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int resourceAcrossFinally(java.lang.String arg0) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `resourceAcrossFinally(Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method resourceAcrossFinally(Ljava/lang/String;)I
        // @declaration a static method of `ScopePlanCrossing`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 27 36 42 52
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
