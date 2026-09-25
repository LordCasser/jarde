package defpackage;

/* JADX INFO: loaded from: AssertProbe.class */
public class AssertProbe {
    private static int guardCalls;
    private static int detailCalls;
    static final /* synthetic */ boolean $assertionsDisabled;

    static {
        $assertionsDisabled = !AssertProbe.class.desiredAssertionStatus();
    }

    private static boolean guard(int value) {
        guardCalls++;
        return value > 0;
    }

    private static String detail(int value) {
        detailCalls++;
        return "bad:" + value;
    }

    static String check(int value) {
        if ($assertionsDisabled || guard(value)) {
            return guardCalls + "|" + detailCalls;
        }
        throw new AssertionError(detail(value));
    }

    public static void main(String[] args) {
        System.out.print(check(1) + ";");
        try {
            System.out.print(check(-1));
        } catch (AssertionError error) {
            System.out.print(error.getMessage() + "|" + guardCalls + "|" + detailCalls);
        }
        System.out.println();
    }
}
