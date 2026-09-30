// Negative 1 (task 1.2, "the lead is no null constant"): the cleanup local's lead reads a
// static field, so the statement before the protected range is `getstatic; astore`, not the
// certificate's `[aconst_null, astore s]`. The copies stay an unconditional same-target call,
// but the null source the slot identity must pin to the lead is absent.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2LeadField {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static java.io.InputStream SOURCE = new java.io.ByteArrayInputStream(new byte[1]);
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { closes++; }
    public Result test(byte[] data) throws IOException {
        InputStream inputStream = SOURCE;
        try {
            inputStream = getInputStream(data);
            decode(inputStream);
            return new Result(400);
        } finally {
            closeQuietly(inputStream);
        }
    }
    public static void main(String[] args) throws Exception {
        Tf2LeadField t = new Tf2LeadField();
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes);
    }
}
