// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally9$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally9$TestCls extends java.lang.Object {
    public TestTryCatchFinally9$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String test() throws java.io.IOException {
        // jarde: not recovered: the recovery run for `test()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test()Ljava/lang/String;
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 33 40 42 47 51 53 59 63
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
