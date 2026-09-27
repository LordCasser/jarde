import java.util.List;

public class FinallyLoop {
    private int count;
    public static boolean fail;

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
    }

    public int count() {
        return count;
    }
}
