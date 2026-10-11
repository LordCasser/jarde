package jadx.tests.integration.switches;

/* JADX INFO: loaded from: TestSwitchSimple$TestCls.class */
public class TestSwitchSimple$TestCls {
    public void test(int a) {
        String s = null;
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
                System.out.println("Not Reach");
                break;
        }
        System.out.println(s);
    }
}