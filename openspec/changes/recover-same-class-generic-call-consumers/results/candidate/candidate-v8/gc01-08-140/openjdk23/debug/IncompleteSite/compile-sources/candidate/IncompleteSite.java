// jarde: presentation of `IncompleteSite` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class IncompleteSite<T> extends java.lang.Object {
    public IncompleteSite() {
        // @method <init>()V
        // @declaration a constructor of `IncompleteSite`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T identity(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `IncompleteSite`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public T relay(T x, boolean use) {
        // jarde: generic Signature `(TT;Z)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method relay(Ljava/lang/Object;Z)Ljava/lang/Object;
        // @declaration an instance method of `IncompleteSite`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (use) {
            return this.identity(x);
        } else {
            return x;
        }
    }
}
