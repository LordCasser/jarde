// jarde: presentation of `StringVarargs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum StringVarargs {
    public static final StringVarargs PAIR;

    public static final StringVarargs SINGLE;

    public static final StringVarargs EMPTY;

    private final java.lang.String[] exts;

    private static final StringVarargs[] $VALUES;

    public static StringVarargs[] values() {
        // @method values()[LStringVarargs;
        // @declaration a static method of `StringVarargs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (StringVarargs[]) StringVarargs.$VALUES.clone();
    }

    public static StringVarargs valueOf(java.lang.String name) {
        // @method valueOf(Ljava/lang/String;)LStringVarargs;
        // @declaration a static method of `StringVarargs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (StringVarargs) java.lang.Enum.valueOf(StringVarargs.class, name);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I[Ljava/lang/String;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private StringVarargs(java.lang.String arg1, int arg2, java.lang.String... extensions) {
        // @method <init>(Ljava/lang/String;I[Ljava/lang/String;)V
        // @declaration a constructor of `StringVarargs`, member flags 0x0082
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.exts = extensions;
        return;
    }

    public java.lang.String[] valuesCopy() {
        // @method valuesCopy()[Ljava/lang/String;
        // @declaration an instance method of `StringVarargs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.exts;
    }

    private static StringVarargs[] $values() {
        // @method $values()[LStringVarargs;
        // @declaration a static method of `StringVarargs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new StringVarargs[]{StringVarargs.PAIR, StringVarargs.SINGLE, StringVarargs.EMPTY};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `StringVarargs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24 21 8 7 11 12 13 15 16 17 18 20
        // the value at BCI 24 comes from an Duplicate at BCI 3, which produces no expression this subset writes
        // @bytecode 27
        // the instruction at BCI 27 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 30
        // the instruction at BCI 30 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 46 43 35 34 38 39 40 42
        // the value at BCI 46 comes from an Duplicate at BCI 30, which produces no expression this subset writes
        // @bytecode 49
        // the instruction at BCI 49 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 52
        // the instruction at BCI 52 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 63 60 57
        // the value at BCI 63 comes from an Duplicate at BCI 52, which produces no expression this subset writes
        StringVarargs.$VALUES = $values();
    }
}
