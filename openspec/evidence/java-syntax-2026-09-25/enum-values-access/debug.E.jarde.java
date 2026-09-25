// jarde: presentation of `E` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum E {
    public static final E A;

    public static final E B;

    private static final E[] $VALUES;

    public static E[] values() {
        // @method values()[LE;
        // @declaration a static method of `E`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (E[]) E.$VALUES.clone();
    }

    public static E valueOf(java.lang.String name) {
        // @method valueOf(Ljava/lang/String;)LE;
        // @declaration a static method of `E`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (E) java.lang.Enum.valueOf(E.class, name);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private E(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `E`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    public static E[] raw() {
        // @method raw()[LE;
        // @declaration a static method of `E`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return E.$VALUES;
    }

    private static E[] $values() {
        // @method $values()[LE;
        // @declaration a static method of `E`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new E[]{E.A, E.B};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `E`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        E.A = new E("A", 0);
        E.B = new E("B", 1);
        E.$VALUES = $values();
    }
}
