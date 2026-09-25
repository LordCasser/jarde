public final class CompoundBoundaryRunner {
    public static void main(String[] args) {
        CompoundBoundaryProbe.reset();
        switch (args[0]) {
            case "field-member":
                CompoundBoundaryProbe.fieldDifferentMember();
                System.out.println("field=" + CompoundBoundaryProbe.box.value
                        + ":other=" + CompoundBoundaryProbe.box.other
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "array-index":
                CompoundBoundaryProbe.arrayDifferentIndex();
                System.out.println("data0=" + CompoundBoundaryProbe.data[0]
                        + ":data1=" + CompoundBoundaryProbe.data[1]
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "array-array":
                CompoundBoundaryProbe.arrayDifferentArray();
                System.out.println("data0=" + CompoundBoundaryProbe.data[0]
                        + ":other0=" + CompoundBoundaryProbe.otherData[0]
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "field-extra":
                CompoundBoundaryProbe.fieldMultiConsumer();
                System.out.println("value=" + CompoundBoundaryProbe.box.value
                        + ":extra=" + CompoundBoundaryProbe.extraConsumers);
                break;
            case "array-extra":
                CompoundBoundaryProbe.arrayMultiConsumer();
                System.out.println("data0=" + CompoundBoundaryProbe.data[0]
                        + ":extra=" + CompoundBoundaryProbe.extraConsumers);
                break;
            case "ordinary-field":
                CompoundBoundaryProbe.ordinaryField();
                System.out.println("value=" + CompoundBoundaryProbe.box.value
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "ordinary-array":
                CompoundBoundaryProbe.ordinaryArray();
                System.out.println("data0=" + CompoundBoundaryProbe.data[0]
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "wide-field":
                CompoundBoundaryProbe.wideField();
                System.out.println("wide=" + CompoundBoundaryProbe.box.wide
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "wide-array":
                CompoundBoundaryProbe.wideArray();
                System.out.println("long0=" + CompoundBoundaryProbe.longData[0]
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "field-snapshot":
                CompoundBoundaryProbe.fieldSnapshot();
                System.out.println("value=" + CompoundBoundaryProbe.box.value
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "array-snapshot":
                CompoundBoundaryProbe.arraySnapshot();
                System.out.println("data0=" + CompoundBoundaryProbe.data[0]
                        + ":select=" + CompoundBoundaryProbe.selects
                        + ":rhs=" + CompoundBoundaryProbe.calls);
                break;
            case "array-null":
                CompoundBoundaryProbe.data = null;
                try {
                    CompoundBoundaryProbe.arrayPrecheck();
                    System.out.println("null-array=returned");
                } catch (NullPointerException expected) {
                    System.out.println("null-array=NPE:select=" + CompoundBoundaryProbe.selects
                            + ":rhs=" + CompoundBoundaryProbe.calls);
                }
                break;
            case "array-bounds":
                CompoundBoundaryProbe.data = new int[1];
                CompoundBoundaryProbe.badIndex = true;
                try {
                    CompoundBoundaryProbe.arrayPrecheck();
                    System.out.println("bounds=returned");
                } catch (ArrayIndexOutOfBoundsException expected) {
                    System.out.println("bounds=AIOOBE:select=" + CompoundBoundaryProbe.selects
                            + ":rhs=" + CompoundBoundaryProbe.calls);
                }
                break;
            default:
                throw new IllegalArgumentException(args[0]);
        }
    }
}
