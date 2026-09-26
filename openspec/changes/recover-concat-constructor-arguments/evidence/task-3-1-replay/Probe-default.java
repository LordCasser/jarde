// jarde: presentation of `Probe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class Probe extends java.lang.Object {
    public Probe() {
        // @method <init>()V
        // @declaration a constructor of `Probe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String thrown(java.lang.String arg0) {
        // @method thrown(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            throw new java.lang.ArithmeticException("arm-" + arg0);
        } catch (java.lang.ArithmeticException local1) {
            return local1.getMessage();
        }
    }

    public static java.lang.String labelThrown(java.lang.String arg0) {
        // @method labelThrown(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            throw new java.lang.ArithmeticException(arg0);
        } catch (java.lang.ArithmeticException local1) {
            return local1.getMessage();
        }
    }

    public static java.lang.String literalThrown() {
        // @method literalThrown()Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            throw new java.lang.ArithmeticException("arm-2");
        } catch (java.lang.ArithmeticException local0) {
            return local0.getMessage();
        }
    }

    public static java.lang.String concatenated(java.lang.String arg0) {
        // @method concatenated(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "arm-" + arg0;
    }

    public static java.lang.ArithmeticException constructed(java.lang.String arg0) {
        // @method constructed(Ljava/lang/String;)Ljava/lang/ArithmeticException;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.ArithmeticException("arm-" + arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) thrown("2"));
        java.lang.System.out.println((java.lang.String) labelThrown("arm-2"));
        java.lang.System.out.println((java.lang.String) literalThrown());
        java.lang.System.out.println((java.lang.String) concatenated("2"));
        java.lang.System.out.println((java.lang.String) constructed("2").getMessage());
        return;
    }
}
