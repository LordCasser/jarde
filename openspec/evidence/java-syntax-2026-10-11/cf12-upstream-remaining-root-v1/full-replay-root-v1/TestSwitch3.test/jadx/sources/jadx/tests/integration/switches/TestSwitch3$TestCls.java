package jadx.tests.integration.switches;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: TestSwitch3$TestCls.class */
public class TestSwitch3$TestCls {
    private int i;

    void test(int a) {
        switch (a) {
            case 1:
                this.i = 1;
                break;
            case 2:
            case 3:
                this.i = 2;
                break;
            default:
                this.i = 4;
                this.i = 5;
                break;
        }
    }

    public void check() {
        test(1);
        JadxAssertions.assertThat(this.i).isEqualTo(1);
        test(2);
        JadxAssertions.assertThat(this.i).isEqualTo(2);
        test(3);
        JadxAssertions.assertThat(this.i).isEqualTo(2);
        test(4);
        JadxAssertions.assertThat(this.i).isEqualTo(5);
        test(10);
        JadxAssertions.assertThat(this.i).isEqualTo(5);
    }
}