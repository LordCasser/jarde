// jarde: presentation of `X1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class X1 extends java.lang.Object {
    public X1() {
        // @method <init>()V
        // @declaration a constructor of `X1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String wrapCtor(java.lang.String arg0) {
        // @method wrapCtor(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `X1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0.isEmpty()) {
                throw new java.lang.IllegalArgumentException("empty");
            } else {
                return arg0;
            }
        } catch (java.lang.IllegalArgumentException local1) {
            java.lang.RuntimeException local2 = new java.lang.RuntimeException("wrapped", (java.lang.Throwable) local1);
            throw local2;
        }
    }

    public static java.lang.String wrapInitCause(java.lang.String arg0) {
        // @method wrapInitCause(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `X1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0.isEmpty()) {
                throw new java.lang.IllegalStateException("bad");
            } else {
                return arg0;
            }
        } catch (java.lang.IllegalStateException local1) {
            local1.initCause((java.lang.Throwable) new java.lang.UnsupportedOperationException("root"));
            throw local1;
        }
    }

    public static java.lang.String readCause(java.lang.Throwable arg0) {
        // @method readCause(Ljava/lang/Throwable;)Ljava/lang/String;
        // @declaration a static method of `X1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Throwable local1 = arg0.getCause();
        return local1 == null ? "none" : local1.getMessage();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `X1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            wrapCtor("");
        } catch (java.lang.RuntimeException local1) {
            java.lang.System.out.println((java.lang.String) readCause((java.lang.Throwable) local1));
        }
        try {
            wrapInitCause("");
        } catch (java.lang.IllegalStateException local1) {
            java.lang.System.out.println((java.lang.String) readCause((java.lang.Throwable) local1));
        }
        // @bytecode 43
        // the instruction at BCI 43 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 46
        // the instruction at BCI 46 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 64 40 61 58
        // the copy at BCI 46 has no proved local assignment
        return;
    }
}
