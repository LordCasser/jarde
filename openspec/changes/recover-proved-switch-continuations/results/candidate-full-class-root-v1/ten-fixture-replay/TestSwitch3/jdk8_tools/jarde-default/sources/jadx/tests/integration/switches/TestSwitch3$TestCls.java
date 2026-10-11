// jarde: presentation of `jadx/tests/integration/switches/TestSwitch3$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitch3$TestCls extends java.lang.Object {
    private int i;

    public TestSwitch3$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitch3$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    void test(int a) {
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch3$TestCls`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        switch (a) {
            case 1:
                this.i = 1;
                return;
            case 2:
            case 3:
                this.i = 2;
                return;
            default:
                this.i = 4;
                this.i = 5;
                return;
        }
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch3$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.test(1);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.i).isEqualTo(1);
        this.test(2);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.i).isEqualTo(2);
        this.test(3);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.i).isEqualTo(2);
        this.test(4);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.i).isEqualTo(5);
        this.test(10);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(this.i).isEqualTo(5);
        return;
    }
}
