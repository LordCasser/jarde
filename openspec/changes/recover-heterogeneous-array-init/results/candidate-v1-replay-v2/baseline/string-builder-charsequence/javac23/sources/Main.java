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
        // @bytecode 8 1 0 4 5 6 9 10 11 14 15 17 20 21 22 25 28 29 32 33 34 37 39 42 43 44 45 48 50 53 54 55 56 59 62 65
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 9 1
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 11
        // the instruction at BCI 11 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 20 1 17
        // the copy at BCI 9 has no proved local assignment
        // @bytecode 21 1
        // the copy at BCI 9 has no proved local assignment
        // @bytecode 22 25 28 29 32 33 34 37 39 42 43 44 45 48 50 53 54 55 56 59 62
        // the statement at BCI 62 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        jarde_refused_body();
    }
}
