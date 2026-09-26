package defpackage;

/* JADX INFO: loaded from: AssertDuplicateStatusWrite.class */
public class AssertDuplicateStatusWrite {
    static final /* synthetic */ boolean $assertionsDisabled;

    private static boolean disabled() {
        return !AssertDuplicateStatusWrite.class.desiredAssertionStatus();
    }

    static {
        $assertionsDisabled = disabled();
        $assertionsDisabled = false;
    }

    static void check(boolean condition) {
        if (!$assertionsDisabled && !condition) {
            throw new AssertionError();
        }
    }
}
