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

    private boolean test(java.lang.Object obj) {
        // @method test(Ljava/lang/Object;)Z
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        boolean res;
        try {
            res = this.exc(obj);
        } catch (java.lang.Exception e) {
            res = false;
        } finally {
            this.f += 1;
        }
        return res;
    }

    private boolean exc(java.lang.Object obj) throws java.lang.Exception {
        // @method exc(Ljava/lang/Object;)Z
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if ("e".equals(obj)) {
            throw new java.lang.Exception("typed");
        } else if (obj instanceof java.lang.AssertionError) {
            throw (java.lang.AssertionError) obj;
    } else {
            return true;
    }
    }
}
