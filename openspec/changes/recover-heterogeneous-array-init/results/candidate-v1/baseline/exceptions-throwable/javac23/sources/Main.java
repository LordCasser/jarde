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
        // @bytecode 15 1 12 0 4 5 6 9 10 16 17 18 21 22 24 27 28 29 32 35 36 39 40 41 44 46 49 50 51 52 55 58 60 63 64 65 66 69 72 75 78
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 16 1
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 18
        // the instruction at BCI 18 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 21
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 27 1 24
        // the copy at BCI 16 has no proved local assignment
        // @bytecode 28 1
        // the copy at BCI 16 has no proved local assignment
        // @bytecode 29 32 35 36 39 40 41 44 46 49 50 51 52 55 58 60 63 64 65 66 69 72 75
        // the statement at BCI 75 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        jarde_refused_body();
    }
}
