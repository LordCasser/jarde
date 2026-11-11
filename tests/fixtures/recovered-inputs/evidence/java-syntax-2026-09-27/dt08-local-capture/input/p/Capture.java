package p;

public class Capture {
    public static Runnable create(final double d) {
        return new Runnable() {
            @Override
            public void run() {
                System.out.println(d);
            }
        };
    }
}
