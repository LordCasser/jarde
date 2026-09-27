// jarde: presentation of `em12/Case` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em12;

public class Case extends em12.Parent {
    public Case() {
        // @method <init>()V
        // @declaration a constructor of `em12.Case`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `em12.Case`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        em12.Case local1 = new em12.Case();
        java.io.PrintStream local2 = java.lang.System.out;
        java.util.Objects.requireNonNull((java.lang.Object) local1);
        // @bytecode 18
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23
        // the instruction at BCI 23 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24 27 22
        // the value at BCI 24 comes from an Duplicate at BCI 23, which produces no expression this subset writes
        // @bytecode 35 32 28 17 22
        // the value at BCI 35 comes from an Duplicate at BCI 21, which produces no expression this subset writes
        return;
    }

    static java.lang.String access$001(em12.Case arg0, em12.Arg arg1) {
        // jarde: not recovered: the recovery run for `access$001(Lem12/Case;Lem12/Arg;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method access$001(Lem12/Case;Lem12/Arg;)Ljava/lang/String;
        // @declaration a static method of `em12.Case`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5 2 0 1
        // the non-constructor invokespecial at BCI 2 is not proven to receive the entry `this`
    }
}
