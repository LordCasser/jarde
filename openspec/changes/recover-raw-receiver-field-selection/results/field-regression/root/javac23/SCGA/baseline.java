// jarde: presentation of `SCGA` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T::Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SCGA<T extends java.lang.Comparable<T>> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at put(Ljava/lang/Comparable;)V@7, main([Ljava/lang/String;)V@30
    public T v;

    public SCGA() {
        // @method <init>()V
        // @declaration a constructor of `SCGA`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void put(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof; same-class call binding proved
        // @method put(Ljava/lang/Comparable;)V
        // @declaration an instance method of `SCGA`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Collections.singletonList((java.lang.Object) arg1);
        this.v = arg1;
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SCGA`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        SCGA local1 = new SCGA();
        local1.put((java.lang.Comparable) "same-class");
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("same-class:").append((java.lang.String) local1.v).toString());
        return;
    }
}
