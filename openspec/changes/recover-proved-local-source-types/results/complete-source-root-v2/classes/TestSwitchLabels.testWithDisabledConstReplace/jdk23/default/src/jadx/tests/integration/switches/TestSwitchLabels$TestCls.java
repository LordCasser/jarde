// jarde: presentation of `jadx/tests/integration/switches/TestSwitchLabels$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitchLabels$TestCls extends java.lang.Object {
    public static final int CONST_ABC = 2748;

    public static final int CONST_CDE = 3294;

    public TestSwitchLabels$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitchLabels$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int f1(int arg0) {
        // @method f1(I)I
        // @declaration a static method of `jadx.tests.integration.switches.TestSwitchLabels$TestCls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case 2748:
                return 3294;
            default:
                return 0;
        }
    }
}
