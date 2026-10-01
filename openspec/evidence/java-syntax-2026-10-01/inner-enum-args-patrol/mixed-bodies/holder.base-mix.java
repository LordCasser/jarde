// jarde: presentation of `p/Holder$Op` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Holder$Op;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Holder$Op {
    public static final p.Holder$Op PLUS;

    public static final p.Holder$Op MUL;

    public static final p.Holder$Op ID;

    private final int k;

    private static final p.Holder$Op[] $VALUES;

    public static p.Holder$Op[] values() {
        // @method values()[Lp/Holder$Op;
        // @declaration a static method of `p.Holder$Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Holder$Op[]) p.Holder$Op.$VALUES.clone();
    }

    public static p.Holder$Op valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)Lp/Holder$Op;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)Lp/Holder$Op;
        // @declaration a static method of `p.Holder$Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 is not part of the provable subset
        // @bytecode 2 3
        // the dependency chain from BCI 3 to final consumer 9 is not bounded
        // @bytecode 2 3 6
        // the dependency chain from BCI 6 to final consumer 9 is not bounded
        // @bytecode 9 6 3 2
        // the value at BCI 9 was produced by a saved declaration this run could not commit
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;II)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Holder$Op(java.lang.String arg1, int arg2, int arg3) {
        // @method <init>(Ljava/lang/String;II)V
        // @declaration a constructor of `p.Holder$Op`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.k = arg3;
        return;
    }

    public int apply(int arg1, int arg2) {
        // @method apply(II)I
        // @declaration an instance method of `p.Holder$Op`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.k;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Holder$Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(p.Holder$Op.PLUS.apply(3, 4));
        java.lang.System.out.println(p.Holder$Op.MUL.apply(3, 4));
        java.lang.System.out.println(p.Holder$Op.ID.apply(3, 4));
        return;
    }

    private static p.Holder$Op[] $values() {
        // @method $values()[Lp/Holder$Op;
        // @declaration a static method of `p.Holder$Op`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Holder$Op[]{p.Holder$Op.PLUS, p.Holder$Op.MUL, p.Holder$Op.ID};
    }

    Holder$Op(java.lang.String arg1, int arg2, int arg3, p.Holder$1 arg4) {
        // @method <init>(Ljava/lang/String;IILp/Holder$1;)V
        // @declaration a constructor of `p.Holder$Op`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, arg3);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `p.Holder$Op`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        p.Holder$Op.PLUS = new p.Holder$Op$1("PLUS", 0, 1);
        p.Holder$Op.MUL = new p.Holder$Op$2("MUL", 1, 2);
        p.Holder$Op.ID = new p.Holder$Op("ID", 2, 0);
        p.Holder$Op.$VALUES = $values();
    }
}
