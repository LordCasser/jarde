// jarde: presentation of `em25/SecondConsumerCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em25;

final class SecondConsumerCases extends java.lang.Object {
    private SecondConsumerCases() {
        // @method <init>()V
        // @declaration a constructor of `em25.SecondConsumerCases`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Integer duplicatedBoxingResult() {
        // @method duplicatedBoxingResult()Ljava/lang/Integer;
        // @declaration a static method of `em25.SecondConsumerCases`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Integer.valueOf(1);
        // @bytecode 4
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the value at BCI 5 comes from an Duplicate at BCI 4, which produces no expression this subset writes
        // @bytecode 6
        // the value at BCI 6 comes from an Duplicate at BCI 4, which produces no expression this subset writes
    }
}
