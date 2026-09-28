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
        // @method test(Ljava/lang/Object;)Z
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        boolean local2;
        try {
            local2 = this.exc(arg1);
        } catch (java.lang.Exception local3) {
            local2 = false;
        } finally {
            this.f += 1;
        }
        return local2;
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
