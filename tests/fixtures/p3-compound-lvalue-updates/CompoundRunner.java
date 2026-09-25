public final class CompoundRunner {
    public static void main(String[] args) {
        CompoundProbe.reset();
        System.out.println("local=" + CompoundProbe.local(5) + ":rhs=" + CompoundProbe.calls);

        CompoundProbe.reset();
        System.out.println("field=" + CompoundProbe.field() + ":select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);

        CompoundProbe.reset();
        System.out.println("array=" + CompoundProbe.array() + ":select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);

        CompoundProbe.reset();
        System.out.println("field-snapshot=" + CompoundProbe.fieldSnapshot() + ":select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);

        CompoundProbe.reset();
        System.out.println("array-snapshot=" + CompoundProbe.arraySnapshot() + ":select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);

        CompoundProbe.reset();
        CompoundProbe.nullBox = true;
        try {
            CompoundProbe.field();
            System.out.println("null=returned");
        } catch (NullPointerException expected) {
            System.out.println("null=NPE:select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);
        }

        CompoundProbe.reset();
        CompoundProbe.badIndex = true;
        try {
            CompoundProbe.array();
            System.out.println("bounds=returned");
        } catch (ArrayIndexOutOfBoundsException expected) {
            System.out.println("bounds=AIOOBE:select=" + CompoundProbe.selects + ":rhs=" + CompoundProbe.calls);
        }
    }
}
