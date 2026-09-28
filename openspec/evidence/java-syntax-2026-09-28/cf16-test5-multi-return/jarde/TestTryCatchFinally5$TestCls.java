// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally5$TestCls extends java.lang.Object {
    public TestTryCatchFinally5$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public java.util.List test(jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls$A a, jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls$B b) {
        // jarde: not recovered: the recovery run for `test(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 12 31 44 53 79 93
        // local 4 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    private jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls$C p(jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls$A a) {
        // @method p(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;)Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$C;
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return (jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls$C) a;
    }
}
