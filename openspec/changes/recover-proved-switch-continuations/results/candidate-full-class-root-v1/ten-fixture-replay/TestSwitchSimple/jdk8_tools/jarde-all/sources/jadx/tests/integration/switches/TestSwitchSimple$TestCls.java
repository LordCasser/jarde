// jarde: presentation of `jadx/tests/integration/switches/TestSwitchSimple$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchSimple$TestCls extends java.lang.Object {
    public TestSwitchSimple$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchSimple$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test(int a) {
        // @method test(I)V
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitchSimple$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String s;
        s = null;
        switch (a % 4) {
            case 1:
                s = "1";
                break;
            case 2:
                s = "2";
                break;
            case 3:
                s = "3";
                break;
            case 4:
                s = "4";
                break;
            default:
                java.lang.System.out.println("Not Reach");
                break;
        }
        java.lang.System.out.println(s);
        return;
    }
}
