// jarde: presentation of `BNX` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BNX extends java.lang.Object {
    public BNX() {
        // @method <init>()V
        // @declaration a constructor of `BNX`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `viaDecimal(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    static java.lang.Number viaDecimal(java.lang.Number arg0, java.lang.Number arg1) {
        // @method viaDecimal(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration a static method of `BNX`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.doubleValue() >= arg1.doubleValue() ? arg0 : arg1;
    }

    // jarde: generic Signature projection refused for `viaAtomic(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    static java.lang.Number viaAtomic(java.lang.Number arg0, java.lang.Number arg1) {
        // @method viaAtomic(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration a static method of `BNX`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.doubleValue() >= arg1.doubleValue() ? arg0 : arg1;
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BNX`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 49 27 46 0 3 6 7 9 12 15 16 18 21 24 30 33 34 35 38 41 42 43 52
        // the parameter 0 of the invocation at BCI 46 is declared `java.lang.Number` presents `java.util.concurrent.atomic.AtomicInteger` but the invocation requires `java.lang.Number` and this layer has no safe reference conversion evidence
        jarde_refused_body();
    }
}
