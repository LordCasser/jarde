// jarde: presentation of `StringParamGetstatic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LStringParamGetstatic;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
enum StringParamGetstatic {
    public static final StringParamGetstatic A;

    private final java.lang.String v;

    private static final StringParamGetstatic[] $VALUES;

    public static StringParamGetstatic[] values() {
        // @method values()[LStringParamGetstatic;
        // @declaration a static method of `StringParamGetstatic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (StringParamGetstatic[]) StringParamGetstatic.$VALUES.clone();
    }

    public static StringParamGetstatic valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LStringParamGetstatic;
        // @declaration a static method of `StringParamGetstatic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (StringParamGetstatic) java.lang.Enum.valueOf(StringParamGetstatic.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;ILjava/lang/String;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private StringParamGetstatic(java.lang.String arg1, int arg2, java.lang.String arg3) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;)V
        // @declaration a constructor of `StringParamGetstatic`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.v = arg3;
        return;
    }

    java.lang.String v() {
        // @method v()Ljava/lang/String;
        // @declaration an instance method of `StringParamGetstatic`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.v;
    }

    private static StringParamGetstatic[] $values() {
        // @method $values()[LStringParamGetstatic;
        // @declaration a static method of `StringParamGetstatic`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new StringParamGetstatic[]{StringParamGetstatic.A};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `StringParamGetstatic`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 13 10 7
        // the copy at BCI 3 has no proved local assignment
        StringParamGetstatic.$VALUES = $values();
    }
}
