// jarde: presentation of `TwoClassVariables` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<A:Ljava/lang/Object;B:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class TwoClassVariables<A, B> extends java.lang.Object {
    public TwoClassVariables() {
        // @method <init>()V
        // @declaration a constructor of `TwoClassVariables`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public B combine(A a, B b) {
        // jarde: generic Signature `(TA;TB;)TB;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method combine(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `TwoClassVariables`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return b;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object a, java.lang.Object b) {
        // @method relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `TwoClassVariables`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.combine(a, b);
    }
}
