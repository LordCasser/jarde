// jarde: presentation of `jadx/tests/integration/switches/TestSwitchWithFallThroughCase$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchWithFallThroughCase$TestCls extends java.lang.Object {
    public TestSwitchWithFallThroughCase$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String test(int a, boolean b, boolean c) {
        // @method test(IZZ)Ljava/lang/String;
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String str;
        str = "";
        switch (a % 4) {
            case 1:
                str = str + ">";
                if (a == 5) {
                    if (b) {
                        if (c) {
                            str = str + "1";
                            break;
                        } else {
                            str = str + "!c";
                            break;
                        }
                    }
                }
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
        str = str + ";";
        return str;
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchWithFallThroughCase$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(5, true, true)).isEqualTo(">1;");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(1, true, true)).isEqualTo(">2;");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(3, true, true)).isEqualTo(";");
        org.assertj.core.api.Assertions.assertThat((java.lang.String) this.test(0, true, true)).isEqualTo("default;");
        return;
    }
}
