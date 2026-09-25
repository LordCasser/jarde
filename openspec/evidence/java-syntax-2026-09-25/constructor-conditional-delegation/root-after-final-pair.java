// jarde: presentation of `ConstructorPairProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ConstructorPairProbe extends java.lang.Object {
    private final java.lang.String first;

    private final java.lang.String second;

    public ConstructorPairProbe(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `ConstructorPairProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg2 == 1 ? arg1 : "", arg2 == 0 ? "" : arg1);
        return;
    }

    public ConstructorPairProbe(java.lang.String arg1, java.lang.String arg2) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `ConstructorPairProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = arg1;
        this.second = arg2;
        return;
    }

    public java.lang.String first() {
        // @method first()Ljava/lang/String;
        // @declaration an instance method of `ConstructorPairProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.first;
    }

    public java.lang.String second() {
        // @method second()Ljava/lang/String;
        // @declaration an instance method of `ConstructorPairProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.second;
    }
}
