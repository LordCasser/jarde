// jarde: presentation of `HeldUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class HeldUse extends java.lang.Object {
    public HeldUse() {
        // @method <init>()V
        // @declaration a constructor of `HeldUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object pick(java.lang.Object arg0, java.lang.Object arg1) {
        // @method pick(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `HeldUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    static java.lang.Object run() {
        // @method run()Ljava/lang/Object;
        // @declaration a static method of `HeldUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0 = new java.lang.StringBuilder("old");
        // @bytecode 11
        // the instruction at BCI 11 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 15
        // the copy at BCI 14 has no proved local assignment
        // @bytecode 18
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the copy at BCI 18 has no proved local assignment
        // @bytecode 23 20 10
        // the value at BCI 23 is the value local 0 held at BCI 10, and the slot does not hold it at BCI 23: the slot's name would read the value the body wrote in between
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `HeldUse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object local1 = run();
        java.lang.System.out.println(local1.getClass().getName() + ":" + local1.toString());
        return;
    }
}
