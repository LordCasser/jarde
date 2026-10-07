// jarde: presentation of `ExceptionChain` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ExceptionChain extends java.lang.Object {
    static boolean result;

    static boolean rhsValue;

    static boolean throwRhs;

    static int calls;

    static int caught;

    public ExceptionChain() {
        // @method <init>()V
        // @declaration a constructor of `ExceptionChain`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `ExceptionChain`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ExceptionChain.calls = ExceptionChain.calls + 1;
        if (ExceptionChain.throwRhs) {
            throw new java.lang.IllegalStateException("rhs");
        } else {
            return ExceptionChain.rhsValue;
        }
    }

    static boolean assign(boolean arg0, boolean arg1) {
        // @method assign(ZZ)Z
        // @declaration a static method of `ExceptionChain`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 0 1 4 5 8 11 14 15 18 19 22
            // block at BCI 8 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        } catch (java.lang.RuntimeException local2) {
            ExceptionChain.caught = ExceptionChain.caught + 1;
        }
        return ExceptionChain.result;
    }

    private static void run(java.lang.String arg0, boolean arg1, boolean arg2, boolean arg3, boolean arg4) {
        // @method run(Ljava/lang/String;ZZZZ)V
        // @declaration a static method of `ExceptionChain`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ExceptionChain.result = false;
        ExceptionChain.rhsValue = arg3;
        ExceptionChain.throwRhs = arg4;
        ExceptionChain.calls = 0;
        ExceptionChain.caught = 0;
        boolean local5 = assign(arg1, arg2);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(arg0).append(":result=").append(local5).append(",calls=").append(ExceptionChain.calls).append(",caught=").append(ExceptionChain.caught).toString());
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ExceptionChain`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        run("extra", true, false, false, false);
        run("left", false, true, false, false);
        run("rhs-true", false, false, true, false);
        run("rhs-false", false, false, false, false);
        run("rhs-throws", false, false, false, true);
        return;
    }
}
