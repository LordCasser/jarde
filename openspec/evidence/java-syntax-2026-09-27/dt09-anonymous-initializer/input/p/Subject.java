package p;

public class Subject {
    public static int value;

    public static Base make() {
        return new Base() {
            {
                value = 1;
            }

            @Override
            public void run() {
                value += 7;
            }
        };
    }
}
