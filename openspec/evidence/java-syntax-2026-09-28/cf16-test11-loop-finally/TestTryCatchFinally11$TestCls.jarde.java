// jarde: presentation of `jadx/tests/integration/trycatch/TestTryCatchFinally11$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.trycatch;

public class TestTryCatchFinally11$TestCls extends java.lang.Object {
    private int count;

    public TestTryCatchFinally11$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.count = 0;
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/util/List;)V`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public void test(java.util.List list) {
        // jarde: not recovered: the recovery run for `test(Ljava/util/List;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljava/util/List;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 10 11 12 17 20 21 26 27 28 29 32 35 38 40 41 46 48 50 55 58 60 65 67 68 70 73 76 78 79
        // the graph is not reducible over 2 block(s) [48, 58]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    private void call1() {
        // @method call1()V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.count += 100;
        return;
    }

    private void call2(java.lang.Object item) {
        // @method call2(Ljava/lang/Object;)V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.count++;
        return;
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls t = new jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls();
        t.test((java.util.List) java.util.Arrays.asList(new java.lang.Object[]{"1", "2"}));
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(t.count).isEqualTo(102);
        return;
    }
}
