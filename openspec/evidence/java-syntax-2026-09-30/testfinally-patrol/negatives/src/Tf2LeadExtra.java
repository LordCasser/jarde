// Negative 2 (task 1.2, "the lead is more than two instructions"): a fresh `Object` anchor
// precedes the null initialisation, so the instructions before the protected range are
// `new; dup; invokespecial; astore; aconst_null; astore` — the null store is no longer the
// two-instruction lead the certificate reads, whatever slot it fills.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2LeadExtra {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { closes++; }
    public Result test(byte[] data) throws IOException {
        Object anchor = new Object();
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
        Tf2LeadExtra t = new Tf2LeadExtra();
        Object anchor = new Object();
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes
            + " anchor=" + (anchor != null));
    }
}
