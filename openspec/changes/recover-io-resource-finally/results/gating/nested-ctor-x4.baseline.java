// jarde: presentation of `X4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class X4 extends java.lang.Object {
    public X4() {
        // @method <init>()V
        // @declaration a constructor of `X4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String threeLayer() {
        // jarde: not recovered: the recovery run for `threeLayer()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method threeLayer()Ljava/lang/String;
        // @declaration a static method of `X4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 26 23 20
        // the copy at BCI 3 has no proved local assignment
    }

    public static java.lang.String doubleUse() {
        // jarde: not recovered: the recovery run for `doubleUse()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method doubleUse()Ljava/lang/String;
        // @declaration a static method of `X4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 12
        // the copy at BCI 9 has no proved local assignment
        // @bytecode 15
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 16
        // the copy at BCI 15 has no proved local assignment
        // @bytecode 23 20 17
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 24 27 28 31 32 35 37 40 41 44 47
        // the statement at BCI 47 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    public static java.lang.String crossBlock(boolean arg0) {
        // jarde: not recovered: the recovery run for `crossBlock(Z)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method crossBlock(Z)Ljava/lang/String;
        // @declaration a static method of `X4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 0 10 22 7
        // the copy at BCI 13 has no proved local assignment
        // @bytecode 31
        // the dependency chain from BCI 31 to final consumer 37 is not bounded
        // @bytecode 31 34
        // the dependency chain from BCI 34 to final consumer 37 is not bounded
        // @bytecode 37 34 31
        // the value at BCI 37 was produced by a saved declaration this run could not commit
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `X4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) threeLayer());
        java.lang.System.out.println((java.lang.String) doubleUse());
        java.lang.System.out.println((java.lang.String) crossBlock(true));
        return;
    }
}
