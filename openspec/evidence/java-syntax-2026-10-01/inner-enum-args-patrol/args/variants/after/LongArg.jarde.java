// jarde: presentation of `LongArg` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LLongArg;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
enum LongArg {
    public static final LongArg A;

    private final long v;

    private static final LongArg[] $VALUES;

    public static LongArg[] values() {
        // @method values()[LLongArg;
        // @declaration a static method of `LongArg`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (LongArg[]) LongArg.$VALUES.clone();
    }

    public static LongArg valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LLongArg;
        // @declaration a static method of `LongArg`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (LongArg) java.lang.Enum.valueOf(LongArg.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IJ)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private LongArg(java.lang.String arg1, int arg2, long arg3) {
        // @method <init>(Ljava/lang/String;IJ)V
        // @declaration a constructor of `LongArg`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.v = arg3;
        return;
    }

    long v() {
        // @method v()J
        // @declaration an instance method of `LongArg`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.v;
    }

    private static LongArg[] $values() {
        // @method $values()[LLongArg;
        // @declaration a static method of `LongArg`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new LongArg[]{LongArg.A};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `LongArg`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        LongArg.A = new LongArg("A", 0, 1L);
        LongArg.$VALUES = $values();
    }
}
