// jarde: presentation of `jadx/tests/integration/switches/TestSwitch$TestCls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package jadx.tests.integration.switches;

public class TestSwitch$TestCls extends java.lang.Object {
    public TestSwitch$TestCls() {
        // @method <init>()V
        // @declaration a constructor of `jadx.tests.integration.switches.TestSwitch$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String test(java.lang.String str) {
        // @method test(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `jadx.tests.integration.switches.TestSwitch$TestCls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int len;
        java.lang.StringBuilder sb;
        int i;
        len = str.length();
        sb = new java.lang.StringBuilder(len);
        for (i = 0; i < len; i = i + 1) {
            char c = str.charAt(i);
            switch (c) {
                case '.':
                case '/':
                    sb.append('_');
                    break;
                case '?':
                    break;
                case ']':
                    sb.append('A');
                    break;
                default:
                    sb.append(c);
                    break;
            }
        }
        return sb.toString();
    }
}
