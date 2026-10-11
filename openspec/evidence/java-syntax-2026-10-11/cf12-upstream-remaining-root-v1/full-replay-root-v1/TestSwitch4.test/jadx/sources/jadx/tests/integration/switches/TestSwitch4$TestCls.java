package jadx.tests.integration.switches;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: TestSwitch4$TestCls.class */
public class TestSwitch4$TestCls {
    private static int parse(char[] ch, int off, int len) {
        int num = ch[(off + len) - 1] - '0';
        switch (len) {
            case 4:
                off++;
                num += (ch[off] - '0') * 1000;
            case 3:
                int i = off;
                off++;
                num += (ch[i] - '0') * 100;
            case 2:
                num += (ch[off] - '0') * 10;
                break;
        }
        return num;
    }

    public void check() {
        JadxAssertions.assertThat(parse("123".toCharArray(), 0, 3)).isEqualTo(123);
        JadxAssertions.assertThat(parse("a=1234".toCharArray(), 2, 4)).isEqualTo(1234);
    }
}