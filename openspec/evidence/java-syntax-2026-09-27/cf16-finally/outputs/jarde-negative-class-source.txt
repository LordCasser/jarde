// jarde: presentation of `TestTryCatchFinally$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class TestTryCatchFinally$TestCls extends java.lang.Object {
    public boolean f;

    public TestTryCatchFinally$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `TestTryCatchFinally$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private boolean test(java.lang.Object arg1) {
        // jarde: not recovered: the recovery run for `test(Ljava/lang/Object;)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(Ljava/lang/Object;)Z
        // @declaration an instance method of `TestTryCatchFinally$TestCls`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 5 6 9 10 11 12 15
        // BCI 31: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 18 19 20 23 24 25 28 31 32 33 34 37 38 39 40 43
        // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [39, 18, 31]
    }

    private static boolean exc(java.lang.Object arg0) throws java.lang.Exception {
        // @method exc(Ljava/lang/Object;)Z
        // @declaration a static method of `TestTryCatchFinally$TestCls`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == null) {
            throw new java.lang.Exception("test");
        } else {
            return arg0 instanceof java.lang.String;
        }
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `TestTryCatchFinally$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.test((java.lang.Object) "a")).isTrue();
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.test((java.lang.Object) null)).isTrue();
        return;
    }
}
