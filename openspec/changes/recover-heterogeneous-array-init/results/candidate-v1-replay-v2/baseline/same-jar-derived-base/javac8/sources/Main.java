// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Main extends java.lang.Object {
    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 13 1 10 0 4 5 6 9 14 15 16 19 20 23 24 25 28 31 32 35 36 37 40 42 45 46 47 48 51 54 56 59 60 61 62 65 68 71 74
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 14 1
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 16
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19
        // the instruction at BCI 19 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 23 1 20
        // the copy at BCI 14 has no proved local assignment
        // @bytecode 24 1
        // the copy at BCI 14 has no proved local assignment
        // @bytecode 25 28 31 32 35 36 37 40 42 45 46 47 48 51 54 56 59 60 61 62 65 68 71
        // the statement at BCI 71 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        jarde_refused_body();
    }
}
