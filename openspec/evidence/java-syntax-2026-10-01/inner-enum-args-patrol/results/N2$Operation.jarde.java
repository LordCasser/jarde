// jarde: presentation of `N2$Operation` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN2$Operation;>;LN2$IOperation;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N2$Operation implements N2$IOperation {
    public static final N2$Operation PLUS;

    public static final N2$Operation MINUS;

    private static final N2$Operation[] $VALUES;

    public static N2$Operation[] values() {
        // @method values()[LN2$Operation;
        // @declaration a static method of `N2$Operation`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (N2$Operation[]) N2$Operation.$VALUES.clone();
    }

    public static N2$Operation valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)LN2$Operation;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)LN2$Operation;
        // @declaration a static method of `N2$Operation`, member flags 0x0009
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

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private N2$Operation(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `N2$Operation`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    private static N2$Operation[] $values() {
        // @method $values()[LN2$Operation;
        // @declaration a static method of `N2$Operation`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new N2$Operation[]{N2$Operation.PLUS, N2$Operation.MINUS};
    }

    N2$Operation(java.lang.String arg1, int arg2, N2$1 arg3) {
        // @method <init>(Ljava/lang/String;ILN2$1;)V
        // @declaration a constructor of `N2$Operation`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `N2$Operation`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        N2$Operation.PLUS = new N2$Operation$1("PLUS", 0);
        N2$Operation.MINUS = new N2$Operation$2("MINUS", 1);
        N2$Operation.$VALUES = $values();
    }
}
