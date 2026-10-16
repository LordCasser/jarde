import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
public class LooplessFinallyProbe {
    static class CleanupFailure extends RuntimeException {}
    static class ObservedOut extends PrintStream {
        int calls;
        final boolean fail;
        ObservedOut(boolean fail) { super(new ByteArrayOutputStream()); this.fail = fail; }
        @Override public void println(String line) {
            if (!"f".equals(line)) throw new AssertionError("unexpected cleanup");
            calls++;
            LooplessFinally.value = 99;
            if (fail) throw new CleanupFailure();
        }
    }
    static void check(String name, int[] xs, boolean fail, int expected, String thrown) {
        PrintStream saved = System.out;
        ObservedOut observed = new ObservedOut(fail);
        LooplessFinally.value = 3000;
        int result = 777;
        String actual = "none";
        try {
            System.setOut(observed);
            result = LooplessFinally.loopless(xs);
        } catch (CleanupFailure ex) {
            actual = "cleanup";
        } catch (NullPointerException ex) {
            actual = "null";
        } finally {
            System.setOut(saved);
        }
        if (result != expected || observed.calls != 1 || LooplessFinally.value != 99 || !actual.equals(thrown))
            throw new AssertionError(name + ":" + result + "/" + observed.calls + "/" + actual);
        saved.println(name + ":" + result + "/" + LooplessFinally.value + "/" + observed.calls + "/" + actual);
    }
    public static void main(String[] args) {
        check("empty", new int[0], false, -1, "none");
        check("two", new int[]{2}, false, -1, "none");
        check("three", new int[]{3}, false, 3000, "none");
        check("null", null, false, 777, "null");
        check("return-throw", new int[]{3}, true, 777, "cleanup");
        check("null-throw", null, true, 777, "cleanup");
    }
}
