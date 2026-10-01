// jarde: presentation of `p/Gs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Gs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Gs {
    public static final p.Gs ONE;

    public static final p.Gs TWO;

    private final p.Simple s;

    private static final p.Gs[] $VALUES;

    public static p.Gs[] values() {
        // @method values()[Lp/Gs;
        // @declaration a static method of `p.Gs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Gs[]) p.Gs.$VALUES.clone();
    }

    public static p.Gs valueOf(java.lang.String arg0) {
        // @method valueOf(Ljava/lang/String;)Lp/Gs;
        // @declaration a static method of `p.Gs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (p.Gs) java.lang.Enum.valueOf(p.Gs.class, arg0);
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/String;ILp/Simple;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    private Gs(java.lang.String arg1, int arg2, p.Simple arg3) {
        // @method <init>(Ljava/lang/String;ILp/Simple;)V
        // @declaration a constructor of `p.Gs`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        this.s = arg3;
        return;
    }

    // jarde: no body: the member `id()I` is declared abstract and its declaration carries no Code attribute
    public abstract int id();

    public p.Simple owner() {
        // @method owner()Lp/Simple;
        // @declaration an instance method of `p.Gs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.s;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Gs`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(p.Gs.ONE.id()).append(" ").append((java.lang.Object) p.Gs.ONE.owner()).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(p.Gs.TWO.id()).append(" ").append((java.lang.Object) p.Gs.TWO.owner()).toString());
        return;
    }

    private static p.Gs[] $values() {
        // @method $values()[Lp/Gs;
        // @declaration a static method of `p.Gs`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Gs[]{p.Gs.ONE, p.Gs.TWO};
    }

    Gs(java.lang.String arg1, int arg2, p.Simple arg3, p.Gs$1 arg4) {
        // @method <init>(Ljava/lang/String;ILp/Simple;Lp/Gs$1;)V
        // @declaration a constructor of `p.Gs`, member flags 0x1000
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, arg2, arg3);
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `p.Gs`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 13 10 7
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 16
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 29 26 23
        // the copy at BCI 19 has no proved local assignment
        p.Gs.$VALUES = $values();
    }
}
