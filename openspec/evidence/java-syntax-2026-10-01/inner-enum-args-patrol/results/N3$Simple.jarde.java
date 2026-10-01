// jarde: presentation of `N3$Simple` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN3$Simple;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N3$Simple {
    public static final N3$Simple A;

    public static final N3$Simple B;

    private final byte num;

    private final java.lang.String s;

    private static final N3$Simple[] $VALUES;

    public static N3$Simple[] values() {
        // @method values()[LN3$Simple;
        // @declaration a static method of `N3$Simple`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (N3$Simple[]) N3$Simple.$VALUES.clone();
    }

    public static N3$Simple valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)LN3$Simple;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)LN3$Simple;
        // @declaration a static method of `N3$Simple`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 is not part of the provable subset
        // @bytecode 2 3
        // the dependency chain from BCI 3 to final consumer 9 is not bounded
        // @bytecode 2 3 6
        // the dependency chain from BCI 6 to final consumer 9 is not bounded
        // @bytecode 9 6 3 2
        // the value at BCI 9 was produced by a saved declaration this run could not commit
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IBLjava/lang/String;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private N3$Simple(java.lang.String arg1, int arg2, byte arg3, java.lang.String arg4) {
        // @method <init>(Ljava/lang/String;IBLjava/lang/String;)V
        // @declaration a constructor of `N3$Simple`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.num = arg3;
        this.s = arg4;
        return;
    }

    public int n() {
        // @method n()I
        // @declaration an instance method of `N3$Simple`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.num;
    }

    private static N3$Simple[] $values() {
        // @method $values()[LN3$Simple;
        // @declaration a static method of `N3$Simple`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new N3$Simple[]{N3$Simple.A, N3$Simple.B};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `N3$Simple`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        N3$Simple.A = new N3$Simple("A", 0, (byte) 1, "x");
        N3$Simple.B = new N3$Simple("B", 1, (byte) 2, "y");
        N3$Simple.$VALUES = $values();
    }
}
