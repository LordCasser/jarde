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

    public static java.lang.String thrown(java.lang.String label) {
        // @method thrown(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 0
            // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
            // @bytecode 3
            // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
            // @bytecode 26 23 16
            // the value at BCI 26 comes from an Duplicate at BCI 3, which produces no expression this subset writes
        } catch (java.lang.ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static java.lang.String labelThrown(java.lang.String label) {
        // @method labelThrown(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            throw new java.lang.ArithmeticException(label);
        } catch (java.lang.ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static java.lang.String literalThrown() {
        // @method literalThrown()Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            throw new java.lang.ArithmeticException("arm-2");
        } catch (java.lang.ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static java.lang.String concatenated(java.lang.String label) {
        // @method concatenated(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "arm-" + label;
    }

    public static java.lang.ArithmeticException constructed(java.lang.String label) {
        // jarde: not recovered: the recovery run for `constructed(Ljava/lang/String;)Ljava/lang/ArithmeticException;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method constructed(Ljava/lang/String;)Ljava/lang/ArithmeticException;
        // @declaration a static method of `Probe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 26 23 16
        // the value at BCI 26 comes from an Duplicate at BCI 3, which produces no expression this subset writes
    }

    public static void main(java.lang.String[] args) {
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
