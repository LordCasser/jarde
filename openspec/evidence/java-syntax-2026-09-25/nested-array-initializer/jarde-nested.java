// jarde: presentation of `NestedArrayInitializer` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedArrayInitializer extends java.lang.Object {
    static int calls;

    static int trace;

    public NestedArrayInitializer() {
        // @method <init>()V
        // @declaration a constructor of `NestedArrayInitializer`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int element(int arg0) {
        // @method element(I)I
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedArrayInitializer.calls = NestedArrayInitializer.calls + 1;
        NestedArrayInitializer.trace = NestedArrayInitializer.trace * 10 + arg0;
        return arg0;
    }

    static int[][] dynamic() {
        // jarde: not recovered: the recovery run for `dynamic()[[I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method dynamic()[[I
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the array instruction at BCI 7 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 9 7
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 15 7 12
        // the value at BCI 15 comes from an Duplicate at BCI 9, which produces no expression this subset writes
        // @bytecode 16 7
        // the instruction at BCI 16 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 22 7 19
        // the value at BCI 22 comes from an Duplicate at BCI 16, which produces no expression this subset writes
        // @bytecode 23 1 7
        // the value at BCI 23 comes from an Duplicate at BCI 4, which produces no expression this subset writes
        // @bytecode 24 1
        // the instruction at BCI 24 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 27
        // the array instruction at BCI 27 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 29 27
        // the instruction at BCI 29 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 35 27 32
        // the value at BCI 35 comes from an Duplicate at BCI 29, which produces no expression this subset writes
        // @bytecode 36 1 27
        // the value at BCI 36 comes from an Duplicate at BCI 24, which produces no expression this subset writes
        // @bytecode 37 1
        // the value at BCI 37 comes from an Duplicate at BCI 24, which produces no expression this subset writes
    }

    static java.lang.String[][] literal() {
        // jarde: not recovered: the recovery run for `literal()[[Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method literal()[[Ljava/lang/String;
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the array instruction at BCI 7 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 10 7
        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 14 7
        // the value at BCI 14 comes from an Duplicate at BCI 10, which produces no expression this subset writes
        // @bytecode 15 7
        // the instruction at BCI 15 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 7
        // the value at BCI 19 comes from an Duplicate at BCI 15, which produces no expression this subset writes
        // @bytecode 20 1 7
        // the value at BCI 20 comes from an Duplicate at BCI 4, which produces no expression this subset writes
        // @bytecode 21 1
        // the instruction at BCI 21 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 24
        // the array instruction at BCI 24 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 27 24
        // the instruction at BCI 27 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 31 24
        // the value at BCI 31 comes from an Duplicate at BCI 27, which produces no expression this subset writes
        // @bytecode 32 1 24
        // the value at BCI 32 comes from an Duplicate at BCI 21, which produces no expression this subset writes
        // @bytecode 33 1
        // the value at BCI 33 comes from an Duplicate at BCI 21, which produces no expression this subset writes
    }
}
