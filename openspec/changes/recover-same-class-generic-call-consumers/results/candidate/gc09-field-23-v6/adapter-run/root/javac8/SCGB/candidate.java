// jarde: presentation of `SCGB` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T::Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SCGB<T extends java.lang.Comparable<T>> extends java.lang.Object {
    // jarde: field Signature `Ljava/util/Map<Ljava/lang/String;Ljava/util/List<TT;>;>;` projected after descriptor erasure and same-class uses at <init>()V@12, main([Ljava/lang/String;)V@24, main([Ljava/lang/String;)V@45
    private java.util.Map<java.lang.String, java.util.List<T>> index;

    public SCGB() {
        // @method <init>()V
        // @declaration a constructor of `SCGB`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.index = new java.util.HashMap();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SCGB`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 8 0 3 4 7 11 14 15 18 20 23 24 27 30 31 34 35 38 41 44 45 48 49 52 55 56 59 61 64 65 68 71 74 77
        // the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 11 14 15
        // the saved producer at BCI 15 has no bounded final expression consumer
        // @bytecode 11 14 15 20
        // the saved producer at BCI 20 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 41 8 38 35 20 15 11 14
        // the value at BCI 41 was produced by a saved declaration this run could not commit
        jarde_refused_body();
    }
}
