package jadx.tests.integration.switches;

/* JADX INFO: loaded from: TestSwitch$TestCls.class */
public class TestSwitch$TestCls {
    public String test(String str) {
        int len = str.length();
        StringBuilder sb = new StringBuilder(len);
        for (int i = 0; i < len; i++) {
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