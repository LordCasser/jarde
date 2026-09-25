public final class CompoundBoundaryProbe {
    public static int calls;
    public static int selects;
    public static int extraConsumers;
    public static int[] data;
    public static int[] otherData;
    public static long[] longData;
    public static BoundaryBox box;
    public static boolean badIndex;

    public static void reset() {
        calls = 0;
        selects = 0;
        extraConsumers = 0;
        badIndex = false;
        data = new int[2];
        data[0] = 10;
        data[1] = 20;
        otherData = new int[2];
        otherData[0] = 30;
        otherData[1] = 40;
        longData = new long[2];
        longData[0] = 50L;
        longData[1] = 60L;
        box = new BoundaryBox(7, 70L);
    }

    public static int rhs(int value) {
        calls++;
        return value;
    }

    public static long longRhs() {
        calls++;
        return 2L;
    }

    public static BoundaryBox receiver() {
        selects++;
        return box;
    }

    public static int index() {
        selects++;
        if (badIndex) {
            return 1;
        }
        return 0;
    }

    public static int rhsFieldMutation() {
        calls++;
        box.value = 100;
        return 2;
    }

    public static int rhsArrayMutation() {
        calls++;
        data[0] = 100;
        return 4;
    }

    public static void observeReceiver(BoundaryBox value) {
        extraConsumers++;
    }

    public static void observeElement(int[] array, int index) {
        extraConsumers++;
    }

    public static void forceObserverReferences() {
        observeReceiver(box);
        observeElement(data, index());
    }

    public static void seedOther() {
        box.other = 1;
    }

    public static void fieldDifferentMember() {
        receiver().value += rhs(2);
    }

    public static void fieldMultiConsumer() {
        receiver().value += rhs(2);
    }

    public static void arrayDifferentIndex() {
        data[index()] += rhs(4);
    }

    public static void arrayDifferentArray() {
        data[index()] += rhs(4);
    }

    public static void arrayMultiConsumer() {
        data[index()] += rhs(4);
    }

    public static void fieldSnapshot() {
        receiver().value += rhsFieldMutation();
    }

    public static void arraySnapshot() {
        data[index()] += rhsArrayMutation();
    }

    public static void arrayPrecheck() {
        data[index()] += rhs(4);
    }

    public static void ordinaryField() {
        receiver().value = rhs(2);
    }

    public static void ordinaryArray() {
        data[index()] = rhs(4);
    }

    public static void wideField() {
        receiver().wide += longRhs();
    }

    public static void wideArray() {
        longData[index()] += longRhs();
    }
}
