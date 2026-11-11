// jarde: presentation of `P02_multianewarray` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class P02_multianewarray extends java.lang.Object {
    // jarde: renamed physical lambda helper "lambda$sum$0" after proving its single class-wide use
    public P02_multianewarray() {
        // @method <init>()V
        // @declaration a constructor of `P02_multianewarray`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int sum(java.util.List arg0) {
        // jarde: generic Signature projection refused for `sum(Ljava/util/List;)I`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
        // jarde: lambda companion call renamed at invokedynamic@17
        // @method sum(Ljava/util/List;)I
        // @declaration a static method of `P02_multianewarray`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[][] local1 = new int[][]{new int[]{0}};
        arg0.forEach((java.util.function.Consumer) ((java.lang.Object p0) -> P02_multianewarray.lambda$sum$0$jarde(local1, (java.lang.Integer) p0)));
        return local1[0][0];
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `P02_multianewarray`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.io.PrintStream saved0 = java.lang.System.out;
        saved0.println(sum((java.util.List) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)})));
        return;
    }

    private static void lambda$sum$0$jarde(int[][] arg0, java.lang.Integer arg1) {
        // jarde: renamed physical lambda helper "lambda$sum$0" to its non-conflicting source name (javac re-synthesizes the physical one beside the lambda expression)
        // jarde: not recovered: the recovery run for `lambda$sum$0([[ILjava/lang/Integer;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method lambda$sum$0([[ILjava/lang/Integer;)V
        // @declaration a static method of `P02_multianewarray`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the array instruction at BCI 2 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4
        // the instruction at BCI 4 is not part of the provable subset
        // @bytecode 2 5 0 1 3 4 6 7 10 11 12
        // the dependency chain from BCI 5 to final consumer 11 is not bounded
        // @bytecode 6 7
        // the dependency chain from BCI 7 to final consumer 11 is not bounded
        // @bytecode 11 2 5 7 0 6
        // the value at BCI 11 comes from an Other at BCI 4, which produces no expression this subset writes
        jarde_refused_body();
    }
}
