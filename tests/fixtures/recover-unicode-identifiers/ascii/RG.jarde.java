// jarde: presentation of `RG` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class RG extends java.lang.Object {
    public RG() {
        // @method <init>()V
        // @declaration a constructor of `RG`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `max(Ljava/lang/Comparable;Ljava/lang/Comparable;)Ljava/lang/Comparable;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    static java.lang.Comparable max(java.lang.Comparable arg0, java.lang.Comparable arg1) {
        // @method max(Ljava/lang/Comparable;Ljava/lang/Comparable;)Ljava/lang/Comparable;
        // @declaration a static method of `RG`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.compareTo((java.lang.Object) arg1) >= 0 ? arg0 : arg1;
    }

    static java.lang.String callGen() {
        // @method callGen()Ljava/lang/String;
        // @declaration a static method of `RG`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) max((java.lang.Comparable) "a", (java.lang.Comparable) "b");
    }

    static java.lang.Integer callGen2() {
        // @method callGen2()Ljava/lang/Integer;
        // @declaration a static method of `RG`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.Integer) max((java.lang.Comparable) java.lang.Integer.valueOf(1), (java.lang.Comparable) java.lang.Integer.valueOf(2));
    }

    // jarde: generic Signature projection refused for `callList()Ljava/util/List;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    static java.util.List callList() {
        // @method callList()Ljava/util/List;
        // @declaration a static method of `RG`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.util.Collections.singletonList((java.lang.Object) "x");
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `RG`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 80 0 77 74 59 54 48 43 37 32 12 29 40 51 71 66 62 3 6 7 10 15 18 19 22 25 26 35 46 57 65 83
        // the parameter 0 of the invocation at BCI 29 is declared `RG$Node` presents `RG$IntNode` but the invocation requires `RG$Node` and this layer has no safe reference conversion evidence
        jarde_refused_body();
    }
}
