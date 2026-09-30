// Patch base (task 1.2, "the saved return's identity is broken"): the fixed lowering itself,
// compiled so the bytecode patch can rewrite the normal completion's `aload_3` (the saved
// `Result`) into `aconst_null` — the return then hands back null, not the value the body
// constructed and stored, and the saved-return identity the certificate proves is gone.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2ReturnIdentity {
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
        Tf2ReturnIdentity t = new Tf2ReturnIdentity();
        Result value = t.test(new byte[1]);
        System.out.println("value=" + (value == null ? "null" : String.valueOf(value.getCode())) + " closes=" + closes);
    }
}
