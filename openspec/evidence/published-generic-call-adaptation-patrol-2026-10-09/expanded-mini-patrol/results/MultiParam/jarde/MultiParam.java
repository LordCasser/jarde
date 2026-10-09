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

    public T first(T x, T y) {
        // jarde: generic Signature `(TT;TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method first(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x, java.lang.Object y) {
        // @method relay(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.first(x, y);
    }

    public static void main(java.lang.String[] a) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MultiParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 8 0 3 4 7 11 14 15 18 20 23 26 27 30 31 32 35 36 39 40 43 44 47 50 53
        // the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 11 14 15
        // the saved producer at BCI 15 has no bounded final expression consumer
        // @bytecode 11 14 15 20
        // the saved producer at BCI 20 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 50 8 47 44 20 15 11 14
        // the value at BCI 50 was produced by a saved declaration this run could not commit
        jarde_refused_body();
    }
}
