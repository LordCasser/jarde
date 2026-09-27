package em23;

/* JADX INFO: loaded from: input.jar:em23/Updates.class */
public class Updates {
    public int instanceField = 1;
    public static int staticField = 1;
    public static String result = "";

    public void increment() {
        this.instanceField++;
    }

    public void decrement() {
        staticField--;
    }

    public void append(String str) {
        result += str + '_';
    }

    public int plusTwo(int i) {
        return i + 2;
    }

    public int next(int i) {
        return i + 1;
    }
}
