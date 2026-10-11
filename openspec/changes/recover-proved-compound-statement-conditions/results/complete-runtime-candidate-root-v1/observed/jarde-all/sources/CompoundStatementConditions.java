// jarde: presentation of `CompoundStatementConditions` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CompoundStatementConditions extends java.lang.Object {
    private CompoundStatementConditions() {
        // @method <init>()V
        // @declaration a constructor of `CompoundStatementConditions`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean probe(java.lang.StringBuilder trace, java.lang.String label, boolean value) {
        // @method probe(Ljava/lang/StringBuilder;Ljava/lang/String;Z)Z
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("test:").append(label).append('=').append(value).append(';');
        return value;
    }

    private static boolean probeOrThrow(java.lang.StringBuilder trace, java.lang.String label, boolean value, java.lang.String throwAt) {
        // @method probeOrThrow(Ljava/lang/StringBuilder;Ljava/lang/String;ZLjava/lang/String;)Z
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("test:").append(label).append('=').append(value).append(';');
        if (label.equals((java.lang.Object) throwAt)) {
            throw new java.lang.IllegalStateException("probe-" + label);
        } else {
            return value;
        }
    }

    private static void appendLabel(java.lang.StringBuilder trace, java.lang.StringBuilder payload, java.lang.String label) {
        // @method appendLabel(Ljava/lang/StringBuilder;Ljava/lang/StringBuilder;Ljava/lang/String;)V
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("emit:").append(label).append(';');
        if (payload.length() != 0) {
            payload.append('>');
        }
        payload.append(label);
        return;
    }

    public static java.lang.String test(boolean a, boolean b, boolean c, boolean d, java.lang.StringBuilder trace) {
        // @method test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder payload;
        payload = new java.lang.StringBuilder();
        appendLabel(trace, payload, "prefix");
        if (probe(trace, "a", a) && probe(trace, "b", b) || probe(trace, "c", c) && probe(trace, "d", d)) {
            appendLabel(trace, payload, "hit");
        } else {
            appendLabel(trace, payload, "miss");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static java.lang.String testMixed(boolean a, boolean b, boolean c, boolean d, java.lang.StringBuilder trace) {
        // @method testMixed(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder payload;
        payload = new java.lang.StringBuilder();
        appendLabel(trace, payload, "prefix");
        if (probe(trace, "a", a) && !probe(trace, "b", b) || !probe(trace, "c", c) && probe(trace, "d", d)) {
            appendLabel(trace, payload, "mixed-miss");
        } else {
            appendLabel(trace, payload, "mixed-hit");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static java.lang.String testThrowing(boolean a, boolean b, boolean c, boolean d, java.lang.String throwAt, java.lang.StringBuilder trace) {
        // @method testThrowing(ZZZZLjava/lang/String;Ljava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder payload;
        payload = new java.lang.StringBuilder();
        appendLabel(trace, payload, "prefix");
        if (probeOrThrow(trace, "a", a, throwAt) && probeOrThrow(trace, "b", b, throwAt) || probeOrThrow(trace, "c", c, throwAt) && probeOrThrow(trace, "d", d, throwAt)) {
            appendLabel(trace, payload, "hit");
        } else {
            appendLabel(trace, payload, "miss");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }

    public static java.lang.String testFloat(float value, java.lang.StringBuilder trace) {
        // @method testFloat(FLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder payload;
        payload = new java.lang.StringBuilder();
        appendLabel(trace, payload, "prefix");
        if (value < 0x0.000000p-126f && !(value < -0x1.400000p3f)) {
            appendLabel(trace, payload, "negative-in-range");
        } else {
            appendLabel(trace, payload, "not-negative-or-below-minus-ten");
        }
        appendLabel(trace, payload, "suffix");
        return payload.toString();
    }
}
