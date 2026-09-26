package defpackage;

/* JADX INFO: loaded from: AssertCore.class */
public class AssertCore {
    private static int guardCalls;
    private static int detailCalls;
    static final /* synthetic */ boolean $assertionsDisabled;

    static {
        $assertionsDisabled = !AssertCore.class.desiredAssertionStatus();
    }

    private static boolean guard(boolean ok) {
        guardCalls++;
        return ok;
    }

    private static String detail() {
        detailCalls++;
        return "bad";
    }

    public static String check(boolean ok) {
        if ($assertionsDisabled || guard(ok)) {
            return guardCalls + "|" + detailCalls;
        }
        throw new AssertionError(detail());
    }

    public static String counts() {
        return guardCalls + "|" + detailCalls;
    }
}
