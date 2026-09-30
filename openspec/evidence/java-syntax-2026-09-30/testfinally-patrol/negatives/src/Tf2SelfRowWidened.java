// Patch base (task 1.2, "the self-protecting row is widened"): the fixed lowering itself,
// compiled so the bytecode patch can widen the handler's own binding row from [32,34) to
// [32,41) — the row then covers the handler cleanup's own call, and a fold that copies the
// cleanup out of the handler would let an exception the widened row catches run it twice.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2SelfRowWidened {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { closes++; }
    public Result test(byte[] data) throws IOException {
        InputStream inputStream = null;
        try {
            inputStream = getInputStream(data);
            decode(inputStream);
            return new Result(400);
        } finally {
            closeQuietly(inputStream);
        }
    }
    public static void main(String[] args) throws Exception {
        Tf2SelfRowWidened t = new Tf2SelfRowWidened();
        Result value = t.test(new byte[1]);
        System.out.println("value=" + (value == null ? "null" : String.valueOf(value.getCode())) + " closes=" + closes);
    }
}
