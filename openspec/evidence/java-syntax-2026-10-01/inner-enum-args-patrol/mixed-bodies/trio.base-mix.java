// jarde: presentation of `p/Trio` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Trio;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Trio {
    public static final p.Trio ONE;

    public static final p.Trio TWO;

    private final int code;

    private final java.lang.String label;

    private final byte small;

    private static final p.Trio[] $VALUES;

    public static p.Trio[] values() {
        // @method values()[Lp/Trio;
        // @declaration a static method of `p.Trio`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Trio[]) p.Trio.$VALUES.clone();
    }

    public static p.Trio valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lp/Trio;
        // @declaration a static method of `p.Trio`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Trio) java.lang.Enum.valueOf(p.Trio.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;IILjava/lang/String;B)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Trio(java.lang.String arg1, int arg2, int arg3, java.lang.String arg4, byte arg5) {
        // @method <init>(Ljava/lang/String;IILjava/lang/String;B)V
        // @declaration a constructor of `p.Trio`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.code = arg3;
        this.label = arg4;
        this.small = arg5;
        return;
    }

    // jarde: no body: the member `describe(I)Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.String describe(int arg1);

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Trio`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.ONE.describe(10)).append(" ").append(p.Trio.ONE.ordinal()).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.TWO.describe(20)).append(" ").append(p.Trio.TWO.ordinal()).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.ONE.name()).append(" ").append((java.lang.String) p.Trio.TWO.name()).toString());
        return;
    }

    private static p.Trio[] $values() {
        // @method $values()[Lp/Trio;
        // @declaration a static method of `p.Trio`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Trio[]{p.Trio.ONE, p.Trio.TWO};
    }

    Trio(java.lang.String arg1, int arg2, int arg3, java.lang.String arg4, byte arg5, p.Trio$1 arg6) {
        // @method <init>(Ljava/lang/String;IILjava/lang/String;BLp/Trio$1;)V
        // @declaration a constructor of `p.Trio`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, arg3, arg4, arg5);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `p.Trio`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        p.Trio.ONE = new p.Trio$1("ONE", 0, 1, "one", (byte) 4);
        p.Trio.TWO = new p.Trio$2("TWO", 1, 2, "two", (byte) 5);
        p.Trio.$VALUES = $values();
    }
}
