// jarde: presentation of `NumberBoundRelay` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
public class NumberBoundRelay<T extends java.lang.Number> extends java.lang.Object {
    public NumberBoundRelay() {
        // @method <init>()V
        // @declaration a constructor of `NumberBoundRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T identity(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method identity(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `NumberBoundRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Number relay(java.lang.Number x) {
        // @method relay(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `NumberBoundRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.identity(x);
    }
}
