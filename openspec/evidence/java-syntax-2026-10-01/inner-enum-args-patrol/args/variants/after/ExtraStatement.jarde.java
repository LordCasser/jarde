// jarde: presentation of `ExtraStatement` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LExtraStatement;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
enum ExtraStatement {
    public static final ExtraStatement A;

    private final byte num;

    private final java.lang.String s;

    private static final ExtraStatement[] $VALUES;

    public static ExtraStatement[] values() {
        // @method values()[LExtraStatement;
        // @declaration a static method of `ExtraStatement`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (ExtraStatement[]) ExtraStatement.$VALUES.clone();
    }

    public static ExtraStatement valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LExtraStatement;
        // @declaration a static method of `ExtraStatement`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (ExtraStatement) java.lang.Enum.valueOf(ExtraStatement.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IBLjava/lang/String;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private ExtraStatement(java.lang.String arg1, int arg2, byte arg3, java.lang.String arg4) {
        // @method <init>(Ljava/lang/String;IBLjava/lang/String;)V
        // @declaration a constructor of `ExtraStatement`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        java.lang.System.out.println("side");
        this.num = arg3;
        this.s = arg4;
        return;
    }

    byte n() {
        // @method n()B
        // @declaration an instance method of `ExtraStatement`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.num;
    }

    private static ExtraStatement[] $values() {
        // @method $values()[LExtraStatement;
        // @declaration a static method of `ExtraStatement`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new ExtraStatement[]{ExtraStatement.A};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `ExtraStatement`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ExtraStatement.A = new ExtraStatement("A", 0, (byte) 1, "x");
        ExtraStatement.$VALUES = $values();
    }
}
