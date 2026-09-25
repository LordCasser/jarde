package defpackage;

/* JADX INFO: loaded from: AssertDifferentConstructor.class */
public class AssertDifferentConstructor {
    static final /* synthetic */ boolean $assertionsDisabled = disabled();

    private static boolean disabled() {
        return !AssertDifferentConstructor.class.desiredAssertionStatus();
    }

    static void check(boolean condition) {
        if (!$assertionsDisabled && !condition) {
            throw new AssertionError(42);
        }
    }
}
