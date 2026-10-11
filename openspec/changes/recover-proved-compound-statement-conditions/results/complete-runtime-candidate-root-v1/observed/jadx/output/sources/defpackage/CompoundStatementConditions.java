package defpackage;

/* JADX INFO: loaded from: CompoundStatementConditions.class */
public final class CompoundStatementConditions {
    private CompoundStatementConditions() {
    }

    private static boolean probe(StringBuilder trace, String label, boolean value) {
        trace.append("test:").append(label).append('=').append(value).append(';');
        return value;
    }

    private static boolean probeOrThrow(StringBuilder trace, String label, boolean value, String throwAt) {
        trace.append("test:").append(label).append('=').append(value).append(';');
        if (label.equals(throwAt)) {
            throw new IllegalStateException("probe-" + label);
        }
        return value;
    }

    private static void appendLabel(StringBuilder trace, StringBuilder payload, String label) {
        trace.append("emit:").append(label).append(';');
        if (payload.length() != 0) {
            payload.append('>');
        }
        payload.append(label);
    }

    public static String test(boolean a, boolean b, boolean c, boolean d, StringBuilder trace) {
        StringBuilder payload = new StringBuilder();
        appendLabel(trace, payload, "prefix");
        if ((probe(trace, "a", a) && probe(trace, "b", b)) || (probe(trace, "c", c) && probe(trace, "d", d))) {
            appendLabel(trace, payload, "hit");
        } else {
            appendLabel(trace, payload, "miss");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static String testMixed(boolean a, boolean b, boolean c, boolean d, StringBuilder trace) {
        StringBuilder payload = new StringBuilder();
        appendLabel(trace, payload, "prefix");
        if ((!probe(trace, "a", a) || probe(trace, "b", b)) && (probe(trace, "c", c) || !probe(trace, "d", d))) {
            appendLabel(trace, payload, "mixed-hit");
        } else {
            appendLabel(trace, payload, "mixed-miss");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static String testThrowing(boolean a, boolean b, boolean c, boolean d, String throwAt, StringBuilder trace) {
        StringBuilder payload = new StringBuilder();
        appendLabel(trace, payload, "prefix");
        if ((probeOrThrow(trace, "a", a, throwAt) && probeOrThrow(trace, "b", b, throwAt)) || (probeOrThrow(trace, "c", c, throwAt) && probeOrThrow(trace, "d", d, throwAt))) {
            appendLabel(trace, payload, "hit");
        } else {
            appendLabel(trace, payload, "miss");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static String testFloat(float value, StringBuilder trace) {
        StringBuilder payload = new StringBuilder();
        appendLabel(trace, payload, "prefix");
        if (value >= 0.0f || value < -10.0f) {
            appendLabel(trace, payload, "not-negative-or-below-minus-ten");
        } else {
            appendLabel(trace, payload, "negative-in-range");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }
}
