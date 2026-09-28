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
        // @method test()Ljava/lang/String;
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.io.InputStream input;
        input = null;
        try {
            input = this.getClass().getResourceAsStream("resource");
            java.util.Scanner scanner = new java.util.Scanner(input).useDelimiter("\\A");
            java.lang.String local3 = scanner.hasNext() ? scanner.next() : "";
            return local3;
        } finally {
            if (input != null) {
                input.close();
            }
        }
    }
}
