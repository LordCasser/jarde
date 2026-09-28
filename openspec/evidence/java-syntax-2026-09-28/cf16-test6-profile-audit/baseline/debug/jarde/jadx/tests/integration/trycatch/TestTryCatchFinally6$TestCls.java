// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally6$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally6$TestCls extends java.lang.Object {
    public TestTryCatchFinally6$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally6$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void test() throws java.io.IOException {
        // jarde: not recovered: the recovery run for `test()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test()V
        // @declaration a static method of `jadx.tests.integration.trycatch.TestTryCatchFinally6$TestCls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 26 31 35 37
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    private static void call() {
        // @method call()V
        // @declaration a static method of `jadx.tests.integration.trycatch.TestTryCatchFinally6$TestCls`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
