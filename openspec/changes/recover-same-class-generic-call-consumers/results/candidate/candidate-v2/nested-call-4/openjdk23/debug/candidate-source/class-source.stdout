// jarde: presentation of `NestedCallArgument` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class NestedCallArgument<T> extends java.lang.Object {
    public NestedCallArgument() {
        // @method <init>()V
        // @declaration a constructor of `NestedCallArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T first(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method first(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `NestedCallArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public T second(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method second(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `NestedCallArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public T relay(T arg1) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `NestedCallArgument`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
                    return this.second(this.first(x));
    }
}
