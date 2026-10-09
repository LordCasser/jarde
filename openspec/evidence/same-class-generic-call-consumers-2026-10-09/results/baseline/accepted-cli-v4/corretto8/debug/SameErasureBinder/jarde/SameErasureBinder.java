// jarde: presentation of `SameErasureBinder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SameErasureBinder<T extends java.lang.Number> extends java.lang.Object {
    public int calls;

    public SameErasureBinder() {
        // @method <init>()V
        // @declaration a constructor of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sink(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public java.lang.Number sink(java.lang.Number x) {
        // @method sink(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.calls += 1;
        return x;
    }

    public T independent(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof
        // @method independent(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public void useNull() {
        // @method useNull()V
        // @declaration an instance method of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.sink((java.lang.Number) null);
        return;
    }
}
