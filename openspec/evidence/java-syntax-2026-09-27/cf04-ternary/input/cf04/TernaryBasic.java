package cf04;

public class TernaryBasic {
    public static int calls;
    private final int value;

    public TernaryBasic(String text, int number) {
        this(text == null ? 0 : number);
    }

    public TernaryBasic(int number) {
        this.value = number;
    }

    public TernaryBasic(String text, int number, boolean marker) {
        this(number == 1 ? text : "", number == 0 ? "" : text);
    }

    public TernaryBasic(String first, String second) {
        this.value = first.length() * 10 + second.length();
    }

    public int value() {
        return value;
    }

    public static int positive(int number) {
        return number > 0 ? number : (number + 2) * 3;
    }

    public static boolean choose(boolean first, boolean second, boolean third) {
        return first ? second : third;
    }

    private static int arm(int value) {
        calls++;
        return value;
    }

    public static int effect(boolean flag) {
        return flag ? arm(1) : arm(2);
    }
}
