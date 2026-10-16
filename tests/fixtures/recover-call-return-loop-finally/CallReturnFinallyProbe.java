import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;

public class CallReturnFinallyProbe {
    static final class CleanupMarker extends RuntimeException { }
    static final class Capture extends PrintStream {
        final List<String> log = new ArrayList<String>();
        final int failAt;
        int calls;
        Capture(int failAt) { super(new ByteArrayOutputStream()); this.failAt = failAt; }
        @Override public void println(String value) {
            log.add(value);
            calls++;
            if (calls == failAt) throw new CleanupMarker();
        }
    }
    static void run(String label, int[] xs, int failAt) {
        PrintStream original = System.out;
        Capture capture = new Capture(failAt);
        String value;
        System.setOut(capture);
        try { value = Integer.toString(CallReturnFinally.finReturn(xs)); }
        catch (CleanupMarker expected) { value = "cleanup"; }
        catch (NullPointerException expected) { value = "null"; }
        finally { System.setOut(original); }
        original.println(label + ":" + value + ":" + capture.calls + ":" + String.join(",", capture.log));
    }
    public static void main(String[] args) {
        run("empty", new int[] {}, 0);
        run("one", new int[] {1}, 0);
        run("two", new int[] {1,2}, 0);
        run("hit", new int[] {3}, 0);
        run("late", new int[] {1,3,5}, 0);
        run("first", new int[] {3,1}, 0);
        run("null", null, 0);
        run("throw-return", new int[] {3}, 1);
        run("throw-late", new int[] {1,3}, 2);
        run("throw-fallthrough", new int[] {1}, 1);
    }
}
