// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally7$TestCls extends java.lang.Object {
    private int f;

    public TestTryCatchFinally7$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.f = 0;
        return;
    }

    private boolean test(java.lang.Object arg1) {
        // jarde: not recovered: the recovery run for `test(Ljava/lang/Object;)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljava/lang/Object;)Z
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 35 50
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    private boolean exc(java.lang.Object arg1) throws java.lang.Exception {
        // @method exc(Ljava/lang/Object;)Z
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if ("r".equals(arg1)) {
            throw new java.lang.AssertionError();
        } else {
            return true;
        }
    }
}
