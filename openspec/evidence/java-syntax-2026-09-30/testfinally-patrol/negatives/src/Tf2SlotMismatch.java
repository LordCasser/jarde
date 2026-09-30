// Negative 3 (task 1.2, "the copies' slot disagrees with the body's assignment"): the lead's
// null and both cleanup copies name `spare`, while the body assigns only the body-local `in` —
// a different slot. The copies are still pairwise identical and read the lead slot, but no body
// assignment of the cleanup slot exists, so the merged value flow the certificate must prove is
// the lead's null alone.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2SlotMismatch {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { closes++; }
    public Result test(byte[] data) throws IOException {
        InputStream spare = null;
        try {
            InputStream in = getInputStream(data);
            decode(in);
            return new Result(400);
        } finally {
            closeQuietly(spare);
        }
    }
    public static void main(String[] args) throws Exception {
        Tf2SlotMismatch t = new Tf2SlotMismatch();
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes);
    }
}
