// jarde: presentation of `F2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class F2 extends java.lang.Object {
    public F2() {
        // @method <init>()V
        // @declaration a constructor of `F2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int stepTwoWithQuote(int[] arg0) {
        // jarde: not recovered: the recovery run for `stepTwoWithQuote([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method stepTwoWithQuote([I)I
        // @declaration a static method of `F2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 28 32 38
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    static int risky(int arg0) {
        // @method risky(I)I
        // @declaration a static method of `F2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == 3) {
            throw new java.lang.IllegalStateException("r");
        } else {
            return arg0;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `F2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(stepTwoWithQuote(local1));
        return;
    }
}
