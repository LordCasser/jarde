// jarde: presentation of `ArrayPostfixElement` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ArrayPostfixElement extends java.lang.Object {
    public ArrayPostfixElement() {
        // @method <init>()V
        // @declaration a constructor of `ArrayPostfixElement`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int[] make(int arg0) {
        // @method make(I)[I
        // @declaration a static method of `ArrayPostfixElement`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 3 1
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 1
        // the value at BCI 6 comes from an Duplicate at BCI 3, which produces no expression this subset writes
        // @bytecode 7 1
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        arg0 = arg0 + 2;
        // @bytecode 13 1 9
        // the value at BCI 13 comes from an Duplicate at BCI 7, which produces no expression this subset writes
        // @bytecode 14 1
        // the instruction at BCI 14 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 19 1
        // the value at BCI 19 comes from an Duplicate at BCI 14, which produces no expression this subset writes
        // @bytecode 20 1
        // the value at BCI 20 comes from an Duplicate at BCI 14, which produces no expression this subset writes
    }
}
