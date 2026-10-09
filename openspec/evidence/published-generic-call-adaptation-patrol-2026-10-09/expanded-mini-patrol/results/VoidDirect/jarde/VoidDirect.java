// jarde: presentation of `VoidDirect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class VoidDirect<T> extends java.lang.Object {
    java.lang.Object seen;

    public VoidDirect() {
        // @method <init>()V
        // @declaration a constructor of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void sink(T x) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof; same-class call binding proved
        // @method sink(Ljava/lang/Object;)V
        // @declaration an instance method of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.seen = x;
        return;
    }

    public void relay(T x) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof; same-class call binding proved
        // @method relay(Ljava/lang/Object;)V
        // @declaration an instance method of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.sink(x);
        return;
    }

    public static void main(java.lang.String[] a) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VoidDirect`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object m = new java.lang.Object();
        VoidDirect c = new VoidDirect();
        c.relay(m);
        java.lang.System.out.println("behavior.marker=" + (c.seen == m));
        return;
    }
}
