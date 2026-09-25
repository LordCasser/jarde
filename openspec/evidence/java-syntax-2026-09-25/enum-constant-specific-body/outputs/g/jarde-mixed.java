// jarde: presentation of `demo/Mixed` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum Mixed {
    public static final demo.Mixed SPECIAL;

    public static final demo.Mixed PLAIN;

    private static final demo.Mixed[] $VALUES;

    public static demo.Mixed[] values() {
        // @method values()[Ldemo/Mixed;
        // @declaration a static method of `demo.Mixed`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Mixed[]) demo.Mixed.$VALUES.clone();
    }

    public static demo.Mixed valueOf(java.lang.String name) {
        // @method valueOf(Ljava/lang/String;)Ldemo/Mixed;
        // @declaration a static method of `demo.Mixed`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Mixed) java.lang.Enum.valueOf(demo.Mixed.class, name);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Mixed(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `demo.Mixed`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `demo.Mixed`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return 0;
    }

    private static demo.Mixed[] $values() {
        // @method $values()[Ldemo/Mixed;
        // @declaration a static method of `demo.Mixed`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new demo.Mixed[]{demo.Mixed.SPECIAL, demo.Mixed.PLAIN};
    }

    Mixed(java.lang.String x0, int x1, demo.Mixed$1 x2) {
        // @method <init>(Ljava/lang/String;ILdemo/Mixed$1;)V
        // @declaration a constructor of `demo.Mixed`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(x0, x1);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `demo.Mixed`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        demo.Mixed.SPECIAL = new demo.Mixed$1("SPECIAL", 0);
        demo.Mixed.PLAIN = new demo.Mixed("PLAIN", 1);
        demo.Mixed.$VALUES = $values();
    }
}
