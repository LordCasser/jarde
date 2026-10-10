package jadx.tests.integration.switches;

/* JADX INFO: loaded from: TestSwitchLabels$TestCls.class */
public class TestSwitchLabels$TestCls {
    public static final int CONST_ABC = 2748;
    public static final int CONST_CDE = 3294;

    /* JADX INFO: loaded from: TestSwitchLabels$TestCls$Inner.class */
    public static class Inner {
        private static final int CONST_CDE_PRIVATE = 3294;

        public int f1(int arg0) {
            switch (arg0) {
                case 3294:
                    return 2748;
                default:
                    return 0;
            }
        }
    }

    public static int f1(int arg0) {
        switch (arg0) {
            case 2748:
                return 3294;
            default:
                return 0;
        }
    }
}