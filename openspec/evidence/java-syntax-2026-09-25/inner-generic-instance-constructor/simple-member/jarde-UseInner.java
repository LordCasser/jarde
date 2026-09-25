// jarde: presentation of `nested/UseInner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package nested;

public final class UseInner extends java.lang.Object {
    public UseInner() {
        // @method <init>()V
        // @declaration a constructor of `nested.UseInner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object make(nested.SimpleOuter arg0, int arg1) {
        // jarde: not recovered: the recovery run for `make(Lnested/SimpleOuter;I)Ljava/lang/Object;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method make(Lnested/SimpleOuter;I)Ljava/lang/Object;
        // @declaration a static method of `nested.UseInner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 9 4
        // the value at BCI 6 comes from an Duplicate at BCI 5, which produces no expression this subset writes
        // @bytecode 19 16 13 4 12
        // the value at BCI 19 comes from an Duplicate at BCI 3, which produces no expression this subset writes
    }
}
