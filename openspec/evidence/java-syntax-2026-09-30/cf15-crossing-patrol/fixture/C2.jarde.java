// jarde: presentation of `C2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class C2 extends java.lang.Object {
    public C2() {
        // @method <init>()V
        // @declaration a constructor of `C2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String alias(int arg0) {
        // @method alias(I)Ljava/lang/String;
        // @declaration a static method of `C2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0 == 0) {
                throw new java.lang.IllegalStateException("zero");
            } else {
                return "fine";
            }
        } catch (java.lang.IllegalStateException local1) {
            java.lang.String local2 = local1.getMessage();
            // @bytecode 50
            // the parameter 1 of the invocation at BCI 47 is declared `java.lang.Throwable` presents `java.lang.IllegalStateException` but the invocation requires `java.lang.Throwable` and this layer has no safe reference conversion evidence
            // @bytecode 51 52
            // the statement at BCI 52 reads `local3`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `C2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            alias(0);
        } catch (java.lang.RuntimeException local1) {
            java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) local1.getMessage()).append("/").append((java.lang.Object) local1.getCause() instanceof java.lang.IllegalStateException).toString());
        }
        java.lang.System.out.println((java.lang.String) alias(1));
        return;
    }
}
