// jarde: presentation of `p/Combo` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Combo;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Combo {
    public static final p.Combo ADD;

    public static final p.Combo MUL;

    public static final p.Combo ID;

    private final int code;

    private static final p.Combo[] $VALUES;

    public static p.Combo[] values() {
        // @method values()[Lp/Combo;
        // @declaration a static method of `p.Combo`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Combo[]) p.Combo.$VALUES.clone();
    }

    public static p.Combo valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lp/Combo;
        // @declaration a static method of `p.Combo`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Combo) java.lang.Enum.valueOf(p.Combo.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;II)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Combo(java.lang.String arg1, int arg2, int arg3) {
        // @method <init>(Ljava/lang/String;II)V
        // @declaration a constructor of `p.Combo`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.code = arg3;
        return;
    }

    public int apply(int arg1, int arg2) {
        // @method apply(II)I
        // @declaration an instance method of `p.Combo`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.code;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Combo`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(p.Combo.ADD.apply(3, 4));
        java.lang.System.out.println(p.Combo.MUL.apply(3, 4));
        java.lang.System.out.println(p.Combo.ID.apply(3, 4));
        return;
    }

    private static p.Combo[] $values() {
        // @method $values()[Lp/Combo;
        // @declaration a static method of `p.Combo`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Combo[]{p.Combo.ADD, p.Combo.MUL, p.Combo.ID};
    }

    Combo(java.lang.String arg1, int arg2, int arg3, p.Combo$1 arg4) {
        // @method <init>(Ljava/lang/String;IILp/Combo$1;)V
        // @declaration a constructor of `p.Combo`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, arg3);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `p.Combo`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        p.Combo.ADD = new p.Combo$1("ADD", 0, 1);
        p.Combo.MUL = new p.Combo$2("MUL", 1, 2);
        p.Combo.ID = new p.Combo("ID", 2, 0);
        p.Combo.$VALUES = $values();
    }
}
