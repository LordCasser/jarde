// jarde: presentation of `N3$Refs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN3$Refs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N3$Refs {
    public static final N3$Refs A;

    public static final N3$Refs B;

    private final N3$Simple s;

    private static final N3$Refs[] $VALUES;

    public static N3$Refs[] values() {
        // @method values()[LN3$Refs;
        // @declaration a static method of `N3$Refs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (N3$Refs[]) N3$Refs.$VALUES.clone();
    }

    public static N3$Refs valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)LN3$Refs;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)LN3$Refs;
        // @declaration a static method of `N3$Refs`, member flags 0x0009
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

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;ILN3$Simple;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private N3$Refs(java.lang.String arg1, int arg2, N3$Simple arg3) {
        // @method <init>(Ljava/lang/String;ILN3$Simple;)V
        // @declaration a constructor of `N3$Refs`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.s = arg3;
        return;
    }

    public N3$Simple g() {
        // @method g()LN3$Simple;
        // @declaration an instance method of `N3$Refs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.s;
    }

    private static N3$Refs[] $values() {
        // @method $values()[LN3$Refs;
        // @declaration a static method of `N3$Refs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new N3$Refs[]{N3$Refs.A, N3$Refs.B};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `N3$Refs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 13 10 7
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 16
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 29 26 23
        // the copy at BCI 19 has no proved local assignment
        N3$Refs.$VALUES = $values();
    }
}
