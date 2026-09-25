public final class CompoundProbe {
    public static int calls;
    public static int selects;
    public static int[] data;
    public static CompoundBox box;
    public static boolean nullBox;
    public static boolean badIndex;

    public static void reset() {
        calls = 0;
        selects = 0;
        data = new int[1];
        data[0] = 10;
        box = new CompoundBox(7);
        nullBox = false;
        badIndex = false;
    }

    public static int rhs(int value) {
        calls++;
        return value;
    }

    public static CompoundBox receiver() {
        selects++;
        if (nullBox) {
            return null;
        }
        return box;
    }

    public static int index() {
        selects++;
        if (badIndex) {
            return 1;
        }
        return 0;
    }

    public static int local(int value) {
        value += rhs(3);
        return value;
    }

    public static int field() {
        receiver().value += rhs(2);
        return box.value;
    }

    public static int array() {
        data[index()] += rhs(4);
        return data[0];
    }

    public static int postField() {
        return receiver().value++;
    }

    public static int postArray() {
        return data[index()]++;
    }
}
