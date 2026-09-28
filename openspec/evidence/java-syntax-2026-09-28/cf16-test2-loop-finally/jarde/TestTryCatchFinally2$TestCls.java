// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally2$TestCls extends java.lang.Object {
    private jadx.core.clsp.ClspClass[] classes;

    public TestTryCatchFinally2$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(java.io.OutputStream output) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `test(Ljava/io/OutputStream;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljava/io/OutputStream;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 35 42 64 76 83 115 122 147 153 160 169
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    private void writeString(java.io.DataOutputStream out, java.lang.String name) {
        // @method writeString(Ljava/io/DataOutputStream;Ljava/lang/String;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }
}
