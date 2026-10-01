// jarde: presentation of `N1$Numbers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN1$Numbers;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N1$Numbers {
    public static final N1$Numbers ONE;

    public static final N1$Numbers TWO;

    private final byte num;

    private final N1$Numbers$NumString str;

    private static final N1$Numbers[] $VALUES;

    public static N1$Numbers[] values() {
        // @method values()[LN1$Numbers;
        // @declaration a static method of `N1$Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (N1$Numbers[]) N1$Numbers.$VALUES.clone();
    }

    public static N1$Numbers valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)LN1$Numbers;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)LN1$Numbers;
        // @declaration a static method of `N1$Numbers`, member flags 0x0009
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

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IBLN1$Numbers$NumString;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private N1$Numbers(java.lang.String arg1, int arg2, byte arg3, N1$Numbers$NumString arg4) {
        // @method <init>(Ljava/lang/String;IBLN1$Numbers$NumString;)V
        // @declaration a constructor of `N1$Numbers`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.num = arg3;
        this.str = arg4;
        return;
    }

    public int getNum() {
        // @method getNum()I
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.num;
    }

    public N1$Numbers$NumString getNumStr() {
        // @method getNumStr()LN1$Numbers$NumString;
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.str;
    }

    public java.lang.String getName() {
        // @method getName()Ljava/lang/String;
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.str.getName();
    }

    private static N1$Numbers[] $values() {
        // @method $values()[LN1$Numbers;
        // @declaration a static method of `N1$Numbers`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new N1$Numbers[]{N1$Numbers.ONE, N1$Numbers.TWO};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `N1$Numbers`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14 11 8
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 17
        // the instruction at BCI 17 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20
        // the instruction at BCI 20 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 31 28 25
        // the copy at BCI 20 has no proved local assignment
        N1$Numbers.$VALUES = $values();
    }
}
