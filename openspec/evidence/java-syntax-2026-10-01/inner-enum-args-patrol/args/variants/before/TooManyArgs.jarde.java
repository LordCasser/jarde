// jarde: presentation of `TooManyArgs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LTooManyArgs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
enum TooManyArgs {
    public static final TooManyArgs A;

    private final byte b;

    private final java.lang.String s;

    private final java.math.BigInteger big;

    private final int i;

    private static final TooManyArgs[] $VALUES;

    public static TooManyArgs[] values() {
        // @method values()[LTooManyArgs;
        // @declaration a static method of `TooManyArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (TooManyArgs[]) TooManyArgs.$VALUES.clone();
    }

    public static TooManyArgs valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LTooManyArgs;
        // @declaration a static method of `TooManyArgs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (TooManyArgs) java.lang.Enum.valueOf(TooManyArgs.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IBLjava/lang/String;Ljava/math/BigInteger;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private TooManyArgs(java.lang.String arg1, int arg2, byte arg3, java.lang.String arg4, java.math.BigInteger arg5, int arg6) {
        // @method <init>(Ljava/lang/String;IBLjava/lang/String;Ljava/math/BigInteger;I)V
        // @declaration a constructor of `TooManyArgs`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.b = arg3;
        this.s = arg4;
        this.big = arg5;
        this.i = arg6;
        return;
    }

    byte b() {
        // @method b()B
        // @declaration an instance method of `TooManyArgs`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.b;
    }

    java.lang.String s() {
        // @method s()Ljava/lang/String;
        // @declaration an instance method of `TooManyArgs`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.s;
    }

    java.math.BigInteger big() {
        // @method big()Ljava/math/BigInteger;
        // @declaration an instance method of `TooManyArgs`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.big;
    }

    int i() {
        // @method i()I
        // @declaration an instance method of `TooManyArgs`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.i;
    }

    private static TooManyArgs[] $values() {
        // @method $values()[LTooManyArgs;
        // @declaration a static method of `TooManyArgs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new TooManyArgs[]{TooManyArgs.A};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `TooManyArgs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 17 14 10
        // the copy at BCI 3 has no proved local assignment
        TooManyArgs.$VALUES = $values();
    }
}
