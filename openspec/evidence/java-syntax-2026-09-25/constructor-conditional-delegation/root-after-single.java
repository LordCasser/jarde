// jarde: presentation of `ConstructorConditionalProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ConstructorConditionalProbe extends java.lang.Object {
    private final int value;

    public ConstructorConditionalProbe(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `ConstructorConditionalProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1 == null ? 0 : arg2);
        return;
    }

    public ConstructorConditionalProbe(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `ConstructorConditionalProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = arg1;
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `ConstructorConditionalProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
