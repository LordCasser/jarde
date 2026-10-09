// jarde: presentation of `IndependentCallee` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class IndependentCallee<T> extends java.lang.Object {
    public IndependentCallee() {
        // @method <init>()V
        // @declaration a constructor of `IndependentCallee`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public <U extends java.lang.Object> U identity(U arg1) {
        // jarde: generic Signature `<U:Ljava/lang/Object;>(TU;)TU;` projected after descriptor erasure and same-run complete AST/Code/SSA formal and return-consumer proof; same-class call binding proved
        // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `IndependentCallee`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    public T relay(T arg1) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `IndependentCallee`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.identity(arg1);
    }
}
