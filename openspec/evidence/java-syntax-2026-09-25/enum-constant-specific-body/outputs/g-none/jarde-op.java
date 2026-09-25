// jarde: presentation of `demo/Op` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum Op {
    public static final demo.Op ADD;

    public static final demo.Op MULTIPLY;

    private static final demo.Op[] $VALUES;

    public static demo.Op[] values() {
        // @method values()[Ldemo/Op;
        // @declaration a static method of `demo.Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Op[]) demo.Op.$VALUES.clone();
    }

    public static demo.Op valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Ldemo/Op;
        // @declaration a static method of `demo.Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (demo.Op) java.lang.Enum.valueOf(demo.Op.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Op(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `demo.Op`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    // jarde: no body: the member `apply(II)I` is declared abstract and its declaration carries no Code attribute
    public abstract int apply(int arg1, int arg2);

    public java.lang.String tag() {
        // @method tag()Ljava/lang/String;
        // @declaration an instance method of `demo.Op`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.name() + ":" + this.ordinal();
    }

    private static demo.Op[] $values() {
        // @method $values()[Ldemo/Op;
        // @declaration a static method of `demo.Op`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new demo.Op[]{demo.Op.ADD, demo.Op.MULTIPLY};
    }

    Op(java.lang.String arg1, int arg2, demo.Op$1 arg3) {
        // @method <init>(Ljava/lang/String;ILdemo/Op$1;)V
        // @declaration a constructor of `demo.Op`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `demo.Op`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        demo.Op.ADD = new demo.Op$1("ADD", 0);
        demo.Op.MULTIPLY = new demo.Op$2("MULTIPLY", 1);
        demo.Op.$VALUES = $values();
    }
}
