public final class BoundaryRunner {
    private static void out(String key, Object value) {
        System.out.println(key + "=" + value);
    }

    public static void main(String[] args) {
        BoundaryProbe.reset();
        String which = args.length == 0 ? "field-member" : args[0];
        if (which.equals("field-member")) {
            int old = BoundaryProbe.fieldDifferentMember();
            out("return.old", old);
            out("sub.value", BoundaryProbe.selected.value);
            out("sub.other", BoundaryProbe.selected.other);
            out("base.value", ((BoundaryBaseBox) BoundaryProbe.selected).value);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("field-owner")) {
            int old = BoundaryProbe.fieldDifferentOwner();
            out("return.old", old);
            out("sub.value", BoundaryProbe.selected.value);
            out("base.value", ((BoundaryBaseBox) BoundaryProbe.selected).value);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("array-index")) {
            int old = BoundaryProbe.arrayDifferentIndex();
            out("return.old", old);
            out("values", BoundaryProbe.values[0] + "," + BoundaryProbe.values[1]);
            out("otherValues", BoundaryProbe.otherValues[0] + "," + BoundaryProbe.otherValues[1]);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("array-array")) {
            int old = BoundaryProbe.arrayDifferentArray();
            out("return.old", old);
            out("values", BoundaryProbe.values[0] + "," + BoundaryProbe.values[1]);
            out("otherValues", BoundaryProbe.otherValues[0] + "," + BoundaryProbe.otherValues[1]);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("field-extra")) {
            int old = BoundaryProbe.fieldExtraConsumer();
            out("return.old", old);
            out("sub.value", BoundaryProbe.selected.value);
            out("observed", BoundaryProbe.observed);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("array-extra")) {
            int old = BoundaryProbe.arrayExtraConsumer();
            out("return.old", old);
            out("values", BoundaryProbe.values[0] + "," + BoundaryProbe.values[1]);
            out("observed", BoundaryProbe.observed);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("field-gap")) {
            int old = BoundaryProbe.fieldGap();
            out("return.old", old);
            out("sub.value", BoundaryProbe.selected.value);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("array-gap")) {
            int old = BoundaryProbe.arrayGap();
            out("return.old", old);
            out("values", BoundaryProbe.values[0] + "," + BoundaryProbe.values[1]);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("prefix")) {
            out("return.new", BoundaryProbe.prefixReceiver());
            out("sub.value", BoundaryProbe.selected.value);
            out("trace", BoundaryProbe.trace);
        } else if (which.equals("assignment")) {
            out("return.assigned", BoundaryProbe.ordinaryAssignment());
            out("sub.value", BoundaryProbe.selected.value);
            out("trace", BoundaryProbe.trace);
        } else {
            throw new IllegalArgumentException(which);
        }
    }
}
