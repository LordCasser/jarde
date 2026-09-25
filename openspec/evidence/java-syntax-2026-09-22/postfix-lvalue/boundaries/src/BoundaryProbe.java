public final class BoundaryProbe {
    public static BoundarySubBox selected;
    public static int[] values;
    public static int[] otherValues;
    public static int selectedIndex;
    public static int observed;
    public static String trace;

    public static void reset() {
        selected = new BoundarySubBox(5, 10, 20);
        values = new int[] { 30, 40 };
        otherValues = new int[] { 50, 60 };
        selectedIndex = 0;
        observed = 0;
        trace = "";
    }

    public static BoundarySubBox receiver() {
        trace += "R";
        return selected;
    }

    public static BoundaryBaseBox baseReceiver() {
        return selected;
    }

    public static int[] array() {
        trace += "A";
        return values;
    }

    public static int index() {
        trace += "I";
        return selectedIndex;
    }

    public static void tick() {
        trace += "T";
    }

    public static void observe(int value) {
        observed += value;
    }

    public static void exposeObserverReference() {
        observe(1);
    }

    public static void exposeTickReference() {
        tick();
    }

    public static int fieldDifferentMember() {
        return receiver().value++;
    }

    public static int fieldOtherMember() {
        return receiver().other++;
    }

    public static int fieldDifferentOwner() {
        return receiver().value++;
    }

    public static int fieldBaseOwner() {
        return baseReceiver().value++;
    }

    public static int fieldGap() {
        return receiver().value++;
    }

    public static int fieldExtraConsumer() {
        return receiver().value++;
    }

    public static int arrayDifferentIndex() {
        return array()[index()]++;
    }

    public static int arrayDifferentArray() {
        return array()[index()]++;
    }

    public static int arrayGap() {
        return array()[index()]++;
    }

    public static int arrayExtraConsumer() {
        return array()[index()]++;
    }

    public static int prefixReceiver() {
        return ++receiver().value;
    }

    public static int ordinaryAssignment() {
        return receiver().value = 99;
    }
}
