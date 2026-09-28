import java.util.List;

public class FinallyLoop {
    private int count;
    public static boolean fail;
    public static boolean failCleanup;

    public void test(List<Object> list) {
        try {
            call1();
        } finally {
            for (Object item : list) {
                call2(item);
            }
        }
    }

    private void call1() {
        count += 100;
        if (fail) {
            throw new IllegalStateException("body");
        }
    }

    private void call2(Object item) {
        count++;
        if (failCleanup) {
            throw new IllegalStateException("call2");
        }
    }

    private void call3(Object item) {
        count += 2;
    }

    public void touch(Object item) {
        call3(item);
    }

    public int count() {
        return count;
    }
}
