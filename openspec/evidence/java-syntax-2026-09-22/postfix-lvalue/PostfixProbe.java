public final class PostfixProbe {
    public static final PostfixProbe INSTANCE = new PostfixProbe();
    public static String trace;
    public static PostfixBox selected;
    public static int[] values;
    public static int selectedIndex;
    public int simpleField;

    public static void reset() {
        trace = "";
        selected = new PostfixBox(41);
        values = new int[] { 70 };
        selectedIndex = 0;
    }

    public static PostfixBox receiver() {
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

    public static int localIncrement(int initial) {
        int value = initial;
        value++;
        return value;
    }

    public int postSimpleField() {
        return this.simpleField++;
    }

    public int preSimpleField() {
        return ++this.simpleField;
    }
}
