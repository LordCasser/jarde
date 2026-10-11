// jarde: presentation of `jadx/tests/integration/switches/TestSwitch4$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitch4$TestCls extends java.lang.Object {
    public TestSwitch4$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitch4$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int parse(char[] ch, int off, int len) {
        // @method parse([CII)I
        // @declaration a static method of `jadx.tests.integration.switches.TestSwitch4$TestCls`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        int num;
        num = ch[off + len - 1] - 48;
        switch (len) {
            case 4:
                num = num + (ch[off++] - 48) * 1000;
            case 3:
                num = num + (ch[off++] - 48) * 100;
            case 2:
                num = num + (ch[off] - 48) * 10;
                break;
        }
        return num;
    }

    public void check() {
        // @method check()V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch4$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(parse((char[]) "123".toCharArray(), 0, 3)).isEqualTo(123);
        jadx.tests.api.utils.assertj.JadxAssertions.assertThat(parse((char[]) "a=1234".toCharArray(), 2, 4)).isEqualTo(1234);
        return;
    }
}
