package em23;

public class Updates {
    public int instanceField = 1;
    public static int staticField = 1;
    public static String result = "";

    public void increment() {
        instanceField++;
    }

    public void decrement() {
        staticField--;
    }

    public void append(String s) {
        result += s + '_';
    }

    public int plusTwo(int value) {
        value += 2;
        return value;
    }

    public int next(int value) {
        value++;
        return value;
    }
}
