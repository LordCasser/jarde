// Patch base (task 1.2, "the rethrow's identity is broken"): the fixed lowering itself,
// compiled so the bytecode patch can rewrite the handler completion's wide `aload 4` (the
// pending throwable) into `aconst_null; nop` — the handler then throws null (a verifier-valid
// NullPointerException at run time), and the original-throwable identity the certificate
// proves is gone.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2RethrowIdentity {
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
        Tf2RethrowIdentity t = new Tf2RethrowIdentity();
        Result value = t.test(new byte[1]);
        System.out.println("value=" + (value == null ? "null" : String.valueOf(value.getCode())) + " closes=" + closes);
    }
}
