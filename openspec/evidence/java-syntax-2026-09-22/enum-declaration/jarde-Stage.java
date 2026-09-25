// jarde: presentation of `Stage` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
enum Stage {
    public static final Stage START;

    public static final Stage FINISH;

    final int ordinalValue;

    private static final Stage[] $VALUES;

    public static Stage[] values() {
        // @method values()[LStage;
        // @declaration a static method of `Stage`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (Stage[]) Stage.$VALUES.clone();
    }

    public static Stage valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)LStage;
        // @declaration a static method of `Stage`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (Stage) java.lang.Enum.valueOf(Stage.class, arg0);
    }

    private Stage(java.lang.String arg1, int arg2, int arg3) {
        // @method <init>(Ljava/lang/String;II)V
        // @declaration a constructor of `Stage`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.ordinalValue = arg3;
        return;
    }

    int code() {
        // @method code()I
        // @declaration an instance method of `Stage`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.ordinalValue;
    }

    private static Stage[] $values() {
        // jarde: not recovered: the recovery run for `$values()[LStage;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method $values()[LStage;
        // @declaration a static method of `Stage`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9 1 6
        // the value at BCI 9 comes from an Duplicate at BCI 4, which produces no expression this subset writes
        // @bytecode 10 1
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 15 1 12
        // the value at BCI 15 comes from an Duplicate at BCI 10, which produces no expression this subset writes
        // @bytecode 16 1
        // the value at BCI 16 comes from an Duplicate at BCI 10, which produces no expression this subset writes
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Stage`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Stage.START = new Stage("START", 0, 4);
        Stage.FINISH = new Stage("FINISH", 1, 9);
        Stage.$VALUES = $values();
    }
}
