// jarde: presentation of `jadx/tests/integration/switches/TestSwitchFallThrough$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchFallThrough$TestCls extends java.lang.Object {
    public int r;

    public TestSwitchFallThrough$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchFallThrough$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(int a) {
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchFallThrough$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int i = 10;
        switch (a) {
            case 1:
                i = 1000;
            case 2:
                this.r = i;
                break;
            default:
                this.r = -1;
                break;
        }
        this.r *= 2;
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("in: ").append(a).append(", out: ").append(this.r).toString());
        return;
    }

    public int testWrap(int a) {
        // @method testWrap(I)I
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchFallThrough$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.r = 0;
        this.test(a);
        return this.r;
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchFallThrough$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.testWrap(1)).isEqualTo(2000);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.testWrap(2)).isEqualTo(20);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.testWrap(0)).isEqualTo(-2);
        return;
    }
}
