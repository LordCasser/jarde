// jarde: presentation of `MethodShadow` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MethodShadow<T> extends java.lang.Object {
    public MethodShadow() {
        // @method <init>()V
        // @declaration a constructor of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public java.lang.Number relay(java.lang.Number x) {
        // @method relay(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.id(x);
    }

    // jarde: generic Signature projection refused for `id(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): method flags, annotations, or existing source refusals cannot be preserved in this generic shape
    public java.lang.Number id(java.lang.Number x) {
        // @method id(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public static void main(java.lang.String[] a) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MethodShadow`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 6 0 2 5 9 12 13 16 18 21 24 25 28 29 32 33 36 37 40 41 44 47 50
        // the saved producer at BCI 6 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 9 12 13
        // the saved producer at BCI 13 has no bounded final expression consumer
        // @bytecode 9 12 13 18
        // the saved producer at BCI 18 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 47 6 44 41 18 13 9 12
        // the value at BCI 47 was produced by a saved declaration this run could not commit
        jarde_refused_body();
    }
}
