// jarde: presentation of `ExceptionShortCircuit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ExceptionShortCircuit extends java.lang.Object {
    static boolean result;

    static int calls;

    public ExceptionShortCircuit() {
        // @method <init>()V
        // @declaration a constructor of `ExceptionShortCircuit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean mayThrow() {
        // @method mayThrow()Z
        // @declaration a static method of `ExceptionShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ExceptionShortCircuit.calls = ExceptionShortCircuit.calls + 1;
        throw new java.lang.IllegalStateException("rhs");
    }

    static boolean assign(boolean arg0) {
        // @method assign(Z)Z
        // @declaration a static method of `ExceptionShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 0 1 4 7 10 11 14 15 18
            // block at BCI 4 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        } catch (java.lang.RuntimeException local1) {
            return ExceptionShortCircuit.result;
        }
        return ExceptionShortCircuit.result;
    }
}
