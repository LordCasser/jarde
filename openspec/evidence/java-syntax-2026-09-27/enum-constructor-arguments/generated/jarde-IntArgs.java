// jarde: presentation of `IntArgs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum IntArgs {
    public static final IntArgs LITERAL;

    public static final IntArgs FIELD;

    public static final IntArgs EXPR;

    private final int n;

    private static final IntArgs[] $VALUES;

    public static IntArgs[] values() {
        // @method values()[LIntArgs;
        // @declaration a static method of `IntArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (IntArgs[]) IntArgs.$VALUES.clone();
    }

    public static IntArgs valueOf(java.lang.String name) {
        // @method valueOf(Ljava/lang/String;)LIntArgs;
        // @declaration a static method of `IntArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (IntArgs) java.lang.Enum.valueOf(IntArgs.class, name);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;II)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private IntArgs(java.lang.String arg1, int arg2, int n) {
        // @method <init>(Ljava/lang/String;II)V
        // @declaration a constructor of `IntArgs`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.n = n;
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `IntArgs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.n;
    }

    private static IntArgs[] $values() {
        // @method $values()[LIntArgs;
        // @declaration a static method of `IntArgs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new IntArgs[]{IntArgs.LITERAL, IntArgs.FIELD, IntArgs.EXPR};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `IntArgs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        IntArgs.LITERAL = new IntArgs("LITERAL", 0, 1);
        // @bytecode 14
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 17
        // the instruction at BCI 17 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 27 24 21
        // the value at BCI 27 comes from an Duplicate at BCI 17, which produces no expression this subset writes
        // @bytecode 30
        // the instruction at BCI 30 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33
        // the instruction at BCI 33 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 45 42 37
        // the value at BCI 45 comes from an Duplicate at BCI 33, which produces no expression this subset writes
        IntArgs.$VALUES = $values();
    }
}
