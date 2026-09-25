// jarde: presentation of `MultiCatchProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MultiCatchProbe extends java.lang.Object {
    static int finallyCalls;

    public MultiCatchProbe() {
        // @method <init>()V
        // @declaration a constructor of `MultiCatchProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String choosePlain(int arg0) {
        // @method choosePlain(I)Ljava/lang/String;
        // @declaration a static method of `MultiCatchProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0 == 1) {
                throw new java.lang.IllegalArgumentException("a");
            } else {
                if (arg0 == 2) {
                    throw new java.lang.IllegalStateException("s");
                } else {
                    // @bytecode 30 32
                    // block at BCI 30 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
                }
            }
        } catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException local1) {
            return local1.getClass().getSimpleName() + ":" + local1.getMessage();
        }
    }

    static java.lang.String chooseFinally(int arg0) {
        // jarde: not recovered: the recovery run for `chooseFinally(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method chooseFinally(I)Ljava/lang/String;
        // @declaration a static method of `MultiCatchProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 5 15 20 30 43 87
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
