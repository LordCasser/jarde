// jarde: presentation of `CallHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class CallHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;)V@10 has no closed SSA source
    public java.lang.Object v;

    public CallHold(T v) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA constructor body proof; same-class call binding proved
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `CallHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = this.identity(v);
        return;
    }

    private T identity(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `CallHold`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }
}
