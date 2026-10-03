// jarde: presentation of `IV3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IV3 extends java.lang.Object {
    private int seed;

    public IV3() {
        // @method <init>()V
        // @declaration a constructor of `IV3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.seed = 2;
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IV3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        IV3 local1 = new IV3();
        java.io.PrintStream saved0 = java.lang.System.out;
        // @bytecode 11
        // the instruction at BCI 11 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 15
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 18
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20
        // the instruction at BCI 20 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21 24 19
        // the copy at BCI 20 has no proved local assignment
        // @bytecode 34 8 31 28 25 19
        // the copy at BCI 14 has no proved local assignment
        return;
    }

    static int access$000(IV3 arg0) {
        // @method access$000(LIV3;)I
        // @declaration a static method of `IV3`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.seed;
    }
}
