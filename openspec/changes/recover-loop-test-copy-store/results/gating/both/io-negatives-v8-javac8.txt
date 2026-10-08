// jarde: presentation of `IONegatives` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class IONegatives extends java.lang.Object {
    private IONegatives() {
        // @method <init>()V
        // @declaration a constructor of `IONegatives`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int twoNested(java.lang.String arg0, java.lang.String arg1) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `twoNested(Ljava/lang/String;Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twoNested(Ljava/lang/String;Ljava/lang/String;)I
        // @declaration a static method of `IONegatives`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 21 29 35 50 59
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int closeReturns(java.lang.String arg0) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `closeReturns(Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method closeReturns(Ljava/lang/String;)I
        // @declaration a static method of `IONegatives`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 11 19 25 34
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
