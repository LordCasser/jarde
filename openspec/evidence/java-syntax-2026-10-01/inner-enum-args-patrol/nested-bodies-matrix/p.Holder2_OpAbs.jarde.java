// jarde: presentation of `p/Holder2$OpAbs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Holder2$OpAbs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Holder2$OpAbs {
    public static final p.Holder2$OpAbs ADD;

    public static final p.Holder2$OpAbs MUL;

    private static final p.Holder2$OpAbs[] $VALUES;

    public static p.Holder2$OpAbs[] values() {
        // @method values()[Lp/Holder2$OpAbs;
        // @declaration a static method of `p.Holder2$OpAbs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Holder2$OpAbs[]) p.Holder2$OpAbs.$VALUES.clone();
    }

    public static p.Holder2$OpAbs valueOf(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `valueOf(Ljava/lang/String;)Lp/Holder2$OpAbs;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method valueOf(Ljava/lang/String;)Lp/Holder2$OpAbs;
        // @declaration a static method of `p.Holder2$OpAbs`, member flags 0x0009
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

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Holder2$OpAbs(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `p.Holder2$OpAbs`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    // jarde: no body: the member `apply(II)I` is declared abstract and its declaration carries no Code attribute
    public abstract int apply(int arg1, int arg2);

    private static p.Holder2$OpAbs[] $values() {
        // @method $values()[Lp/Holder2$OpAbs;
        // @declaration a static method of `p.Holder2$OpAbs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Holder2$OpAbs[]{p.Holder2$OpAbs.ADD, p.Holder2$OpAbs.MUL};
    }

    Holder2$OpAbs(java.lang.String arg1, int arg2, p.Holder2$1 arg3) {
        // @method <init>(Ljava/lang/String;ILp/Holder2$1;)V
        // @declaration a constructor of `p.Holder2$OpAbs`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `p.Holder2$OpAbs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        p.Holder2$OpAbs.ADD = new p.Holder2$OpAbs$1("ADD", 0);
        p.Holder2$OpAbs.MUL = new p.Holder2$OpAbs$2("MUL", 1);
        p.Holder2$OpAbs.$VALUES = $values();
    }
}
