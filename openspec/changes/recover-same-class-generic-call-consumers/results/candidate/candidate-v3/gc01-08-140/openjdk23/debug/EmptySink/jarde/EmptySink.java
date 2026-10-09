// jarde: presentation of `EmptySink` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class EmptySink<T> extends java.lang.Object {
    public EmptySink() {
        // @method <init>()V
        // @declaration a constructor of `EmptySink`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void sink(T x) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method sink(Ljava/lang/Object;)V
        // @declaration an instance method of `EmptySink`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
