// jarde: presentation of `NestedDepth` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedDepth extends java.lang.Object {
    private NestedDepth() {
        // @method <init>()V
        // @declaration a constructor of `NestedDepth`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String threeLayer() {
        // jarde: not recovered: the recovery run for `threeLayer()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method threeLayer()Ljava/lang/String;
        // @declaration a static method of `NestedDepth`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 32 29 26 23 20
        // the copy at BCI 3 has no proved local assignment
    }

    public static java.lang.String fourLayer() {
        // jarde: not recovered: the recovery run for `fourLayer()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method fourLayer()Ljava/lang/String;
        // @declaration a static method of `NestedDepth`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 4
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 42 39 36 33 30 27 24
        // the copy at BCI 3 has no proved local assignment
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NestedDepth`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) threeLayer());
        java.lang.System.out.println((java.lang.String) fourLayer());
        return;
    }
}
