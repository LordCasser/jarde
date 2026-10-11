package jadx.tests.integration.switches;

import org.assertj.core.api.Assertions;

/* JADX INFO: loaded from: TestSwitchWithFallThroughCase$TestCls.class */
public class TestSwitchWithFallThroughCase$TestCls {
    /* JADX WARN: Code duplicated, block: B:14:0x0079  */
    public String test(int a, boolean b, boolean c) {
        String str = "";
        switch (a % 4) {
            case 1:
                str = str + ">";
                if (a == 5 && b) {
                    if (c) {
                        str = str + "1";
                    } else {
                        str = str + "!c";
                    }
                } else if (b) {
                    str = str + "2";
                }
                break;
            case 2:
                if (b) {
                    str = str + "2";
                }
                break;
            case 3:
                break;
            default:
                str = str + "default";
                break;
        }
        return str + ";";
    }

    public void check() {
        Assertions.assertThat(test(5, true, true)).isEqualTo(">1;");
        Assertions.assertThat(test(1, true, true)).isEqualTo(">2;");
        Assertions.assertThat(test(3, true, true)).isEqualTo(";");
        Assertions.assertThat(test(0, true, true)).isEqualTo("default;");
    }
}