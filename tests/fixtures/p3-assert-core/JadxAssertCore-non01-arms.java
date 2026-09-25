package defpackage;

/* JADX INFO: loaded from: AssertCore-non01-arms.class */
public class AssertCore {
    private static int guardCalls;
    private static int detailCalls;
    static final /* synthetic */ boolean $assertionsDisabled;

    /* JADX WARN: Multi-variable type inference failed */
    /* JADX WARN: Type inference failed for: r0v2 */
    /* JADX WARN: Type inference failed for: r0v4 */
    static {
        $assertionsDisabled = !AssertCore.class.desiredAssertionStatus() ? 2 : 3;
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
