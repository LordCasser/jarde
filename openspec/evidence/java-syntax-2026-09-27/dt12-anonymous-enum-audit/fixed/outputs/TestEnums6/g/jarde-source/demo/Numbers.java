// jarde: presentation of `demo/Numbers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum Numbers {
    public static final demo.Numbers ZERO;

    public static final demo.Numbers ONE;

    private final int n;

    private static final demo.Numbers[] $VALUES;

    public static demo.Numbers[] values() {
        // @method values()[Ldemo/Numbers;
        // @declaration a static method of `demo.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Numbers[]) demo.Numbers.$VALUES.clone();
    }

    public static demo.Numbers valueOf(java.lang.String name) {
        // @method valueOf(Ljava/lang/String;)Ldemo/Numbers;
        // @declaration a static method of `demo.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Numbers) java.lang.Enum.valueOf(demo.Numbers.class, name);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Numbers(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `demo.Numbers`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, 0);
        return;
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;II)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Numbers(java.lang.String arg1, int arg2, int n) {
        // @method <init>(Ljava/lang/String;II)V
        // @declaration a constructor of `demo.Numbers`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.n = n;
        return;
    }

    public int getN() {
        // @method getN()I
        // @declaration an instance method of `demo.Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.n;
    }

    private static demo.Numbers[] $values() {
        // @method $values()[Ldemo/Numbers;
        // @declaration a static method of `demo.Numbers`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new demo.Numbers[]{demo.Numbers.ZERO, demo.Numbers.ONE};
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `demo.Numbers`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        demo.Numbers.ZERO = new demo.Numbers("ZERO", 0);
        demo.Numbers.ONE = new demo.Numbers("ONE", 1, 1);
        demo.Numbers.$VALUES = $values();
    }
}
