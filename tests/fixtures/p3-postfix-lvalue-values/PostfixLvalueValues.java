public final class PostfixLvalueValues {
    public static String trace;
    public static PostfixLvalueValues selected;
    public static int[] values;
    public static int selectedIndex;
    public int value;

    public static void reset() {
        trace = "";
        selected = new PostfixLvalueValues();
        selected.value = 41;
        values = new int[] { 70 };
        selectedIndex = 0;
    }

    public static PostfixLvalueValues receiver() {
        trace += "R";
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

    public static int postReceiver() {
        return receiver().value++;
    }

    public static int postArray() {
        return array()[index()]++;
    }

    public int postSimpleField() {
        return this.value++;
    }

    public int preSimpleField() {
        return ++this.value;
    }
}
