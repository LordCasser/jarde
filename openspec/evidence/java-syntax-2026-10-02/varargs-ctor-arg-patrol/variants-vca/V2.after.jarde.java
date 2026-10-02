// jarde: presentation of `V2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V2 extends java.lang.Object {
    static int kept;

    static java.lang.Integer[] keptArr;

    public V2() {
        // @method <init>()V
        // @declaration a constructor of `V2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int midStatement() {
        // jarde: not recovered: the recovery run for `midStatement()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method midStatement()I
        // @declaration a static method of `V2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the array instruction at BCI 5 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 8 5
        // the instruction at BCI 8 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14 5 11
        // the copy at BCI 8 has no proved local assignment
        // @bytecode 15 5
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 18
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the copy at BCI 18 has no proved local assignment
        // @bytecode 25 5 22
        // the copy at BCI 15 has no proved local assignment
        // @bytecode 35 32 29 26 5
        // the copy at BCI 3 has no proved local assignment
    }

    public static int doubleUse() {
        // jarde: not recovered: the recovery run for `doubleUse()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method doubleUse()I
        // @declaration a static method of `V2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the array instruction at BCI 5 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 8 5
        // the instruction at BCI 8 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14 5 11
        // the copy at BCI 8 has no proved local assignment
        // @bytecode 15 5
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21 5 18
        // the copy at BCI 15 has no proved local assignment
        // @bytecode 22 5
        // the instruction at BCI 22 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23 5
        // the copy at BCI 22 has no proved local assignment
        // @bytecode 35 32 29 26 5
        // the copy at BCI 3 has no proved local assignment
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(midStatement()).append(":").append(doubleUse()).append(":").append(V2.kept).append(":").append((java.lang.Object) V2.keptArr[0]).toString());
        return;
    }
}
