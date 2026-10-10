package jadx.tests.integration.switches;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: TestSwitchFallThrough$TestCls.class */
public class TestSwitchFallThrough$TestCls {
    public int r;

    public void test(int a) {
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
        System.out.println("in: " + a + ", out: " + this.r);
    }

    public int testWrap(int a) {
        this.r = 0;
        test(a);
        return this.r;
    }

    public void check() {
        JadxAssertions.assertThat(testWrap(1)).isEqualTo(2000);
        JadxAssertions.assertThat(testWrap(2)).isEqualTo(20);
        JadxAssertions.assertThat(testWrap(0)).isEqualTo(-2);
    }
}