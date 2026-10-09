// jarde: presentation of `MultiParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MultiParam<T> extends java.lang.Object {
    public MultiParam() {
        // @method <init>()V
        // @declaration a constructor of `MultiParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T first(T arg1, T arg2) {
        // jarde: generic Signature `(TT;TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method first(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object arg1, java.lang.Object arg2) {
        // @method relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.first(arg1, arg2);
    }
}
