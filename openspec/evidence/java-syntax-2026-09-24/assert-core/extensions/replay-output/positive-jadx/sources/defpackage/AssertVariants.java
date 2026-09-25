package defpackage;

/* JADX INFO: loaded from: AssertVariants.class */
public class AssertVariants {
    static int effects;
    static final /* synthetic */ boolean $assertionsDisabled;

    static {
        $assertionsDisabled = !AssertVariants.class.desiredAssertionStatus();
        effects++;
    }

    private static boolean probe(boolean value) {
        effects = (effects * 10) + 2;
        return value;
    }

    private static String detail() {
        effects = (effects * 10) + 3;
        return "message";
    }

    static int noMessage(boolean condition) {
        if ($assertionsDisabled || probe(condition)) {
            return effects;
        }
        throw new AssertionError();
    }

    static int multiple(boolean first, boolean second) {
        if (!$assertionsDisabled && !probe(first)) {
            throw new AssertionError();
        }
        if ($assertionsDisabled || probe(second)) {
            return effects;
        }
        throw new AssertionError(detail());
    }
}
