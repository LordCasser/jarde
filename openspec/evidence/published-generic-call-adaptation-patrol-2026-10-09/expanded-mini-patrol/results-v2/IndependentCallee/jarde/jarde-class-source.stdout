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

    // jarde: generic Signature projection refused for `id(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (generic_source_shape_unproved): method flags, annotations, or existing source refusals cannot be preserved in this generic shape
    public java.lang.Object id(java.lang.Object x) {
        // @method id(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `IndependentCallee`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `IndependentCallee`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.id(x);
    }

    public static void main(java.lang.String[] a) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IndependentCallee`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 8 0 3 4 7 11 14 15 18 20 23 26 27 30 31 34 35 38 39 42 43 46 49 52
        // the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 11 14 15
        // the saved producer at BCI 15 has no bounded final expression consumer
        // @bytecode 11 14 15 20
        // the saved producer at BCI 20 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 49 8 46 43 20 15 11 14
        // the value at BCI 49 was produced by a saved declaration this run could not commit
        jarde_refused_body();
    }
}
