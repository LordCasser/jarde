// Negative 5 (task 1.2, "copy grammar addition"): the cleanup carries a second statement after
// the call, so each copy is `aload; aload; invokespecial; invokestatic` — two calls where the
// certificate proves exactly one, and the fold would have to drop a real effect.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2CleanupExtra {
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
            Counter.count();
        }
    }
    public static void main(String[] args) throws Exception {
        Tf2CleanupExtra t = new Tf2CleanupExtra();
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes);
    }
}

class Counter {
    static void count() { Tf2CleanupExtra.closes += 100; }
}
