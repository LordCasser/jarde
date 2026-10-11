package defpackage;

/* JADX INFO: loaded from: CompoundConditionNegative.class */
public final class CompoundConditionNegative {
    private CompoundConditionNegative() {
    }

    /* JADX WARN: Code duplicated, block: B:10:0x0020 A[PHI: r7
      0x0020: PHI (r7v3 'x' int) = (r7v1 'x' int), (r7v4 'x' int) binds: [B:9:0x001d, B:5:0x0015] A[DONT_GENERATE, DONT_INLINE]] */
    /* JADX WARN: Code duplicated, block: B:6:0x0018 A[PHI: r7
      0x0018: PHI (r7v1 'x' int) = (r7v0 'x' int), (r7v4 'x' int) binds: [B:3:0x000d, B:5:0x0015] A[DONT_GENERATE, DONT_INLINE]] */
    public static String prefixIncrement(boolean a, int initial, boolean c, boolean d) {
        int x = initial;
        StringBuilder payload = new StringBuilder();
        if (a) {
            x++;
            if (x > 0) {
                payload.append("hit");
            } else if (!c && d) {
                payload.append("hit");
            } else {
                payload.append("miss");
            }
        } else {
            if (!c) {
            }
            payload.append("miss");
        }
        return payload.toString() + ":" + x;
    }

    /* JADX WARN: Code duplicated, block: B:10:0x0022 A[PHI: r7
      0x0022: PHI (r7v3 'x' int) = (r7v1 'x' int), (r7v4 'x' int) binds: [B:9:0x001f, B:5:0x0017] A[DONT_GENERATE, DONT_INLINE]] */
    /* JADX WARN: Code duplicated, block: B:6:0x001a A[PHI: r7
      0x001a: PHI (r7v1 'x' int) = (r7v0 'x' int), (r7v4 'x' int) binds: [B:3:0x000d, B:5:0x0017] A[DONT_GENERATE, DONT_INLINE]] */
    public static String localAssignment(boolean a, int initial, boolean c, boolean d) {
        int x = initial;
        StringBuilder payload = new StringBuilder();
        if (a) {
            int i = x + 1;
            x = i;
            if (i > 0) {
                payload.append("hit");
            } else if (!c && d) {
                payload.append("hit");
            } else {
                payload.append("miss");
            }
        } else {
            if (!c) {
            }
            payload.append("miss");
        }
        return payload.toString() + ":" + x;
    }
}
