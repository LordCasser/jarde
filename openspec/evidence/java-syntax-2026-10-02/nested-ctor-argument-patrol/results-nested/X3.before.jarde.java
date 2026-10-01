// jarde: presentation of `X3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class X3 extends java.lang.Object {
    public X3() {
        // @method <init>()V
        // @declaration a constructor of `X3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String doubleNested() {
        // jarde: not recovered: the recovery run for `doubleNested()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method doubleNested()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 28 25 22
        // the copy at BCI 3 has no proved local assignment
    }

    public static java.lang.String secondPosition() {
        // jarde: not recovered: the recovery run for `secondPosition()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method secondPosition()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21 18 15
        // the copy at BCI 3 has no proved local assignment
    }

    public static java.lang.String sameClassTwice() {
        // jarde: not recovered: the recovery run for `sameClassTwice()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sameClassTwice()Ljava/lang/String;
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 28 25 22
        // the copy at BCI 3 has no proved local assignment
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `X3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) doubleNested());
        java.lang.System.out.println((java.lang.String) secondPosition());
        java.lang.System.out.println((java.lang.String) sameClassTwice());
        return;
    }
}
