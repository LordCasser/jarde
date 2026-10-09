// jarde: presentation of `NonzeroHeldUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NonzeroHeldUse extends java.lang.Object {
    public NonzeroHeldUse() {
        // @method <init>()V
        // @declaration a constructor of `NonzeroHeldUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object pick(java.lang.Object arg0, java.lang.Object arg1, java.lang.Object arg2) {
        // @method pick(Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `NonzeroHeldUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    static java.lang.Object run() {
        // @method run()Ljava/lang/Object;
        // @declaration a static method of `NonzeroHeldUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0 = new java.lang.StringBuilder("old");
        java.lang.StringBuilder saved0 = (java.lang.StringBuilder) local0;
        // @bytecode 16
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20
        // the copy at BCI 19 has no proved local assignment
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the copy at BCI 23 has no proved local assignment
        // @bytecode 28 25 13 12
        // the copy at BCI 23 has no proved local assignment
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NonzeroHeldUse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object local1 = run();
        java.lang.System.out.println(local1.getClass().getName() + ":" + local1.toString());
        return;
    }
}
