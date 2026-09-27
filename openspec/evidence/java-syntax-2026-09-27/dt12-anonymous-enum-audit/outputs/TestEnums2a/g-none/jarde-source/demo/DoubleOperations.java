// jarde: presentation of `demo/DoubleOperations` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum DoubleOperations implements demo.IOps {
    public static final demo.DoubleOperations TIMES;

    public static final demo.DoubleOperations DIVIDE;

    private final java.lang.String op;

    private static final demo.DoubleOperations[] $VALUES;

    public static demo.DoubleOperations[] values() {
        // @method values()[Ldemo/DoubleOperations;
        // @declaration a static method of `demo.DoubleOperations`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.DoubleOperations[]) demo.DoubleOperations.$VALUES.clone();
    }

    public static demo.DoubleOperations valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Ldemo/DoubleOperations;
        // @declaration a static method of `demo.DoubleOperations`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.DoubleOperations) java.lang.Enum.valueOf(demo.DoubleOperations.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;ILjava/lang/String;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private DoubleOperations(java.lang.String arg1, int arg2, java.lang.String arg3) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;)V
        // @declaration a constructor of `demo.DoubleOperations`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.op = arg3;
        return;
    }

    public java.lang.String getOp() {
        // @method getOp()Ljava/lang/String;
        // @declaration an instance method of `demo.DoubleOperations`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.op;
    }

    private static demo.DoubleOperations[] $values() {
        // @method $values()[Ldemo/DoubleOperations;
        // @declaration a static method of `demo.DoubleOperations`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new demo.DoubleOperations[]{demo.DoubleOperations.TIMES, demo.DoubleOperations.DIVIDE};
    }

    DoubleOperations(java.lang.String arg1, int arg2, java.lang.String arg3, demo.DoubleOperations$1 arg4) {
        // @method <init>(Ljava/lang/String;ILjava/lang/String;Ldemo/DoubleOperations$1;)V
        // @declaration a constructor of `demo.DoubleOperations`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, arg3);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `demo.DoubleOperations`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        demo.DoubleOperations.TIMES = new demo.DoubleOperations$1("TIMES", 0, "*");
        demo.DoubleOperations.DIVIDE = new demo.DoubleOperations$2("DIVIDE", 1, "/");
        demo.DoubleOperations.$VALUES = $values();
    }
}
