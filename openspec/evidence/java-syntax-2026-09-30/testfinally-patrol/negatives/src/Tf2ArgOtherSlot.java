// Negative 4 (task 1.2, "the copies' argument reads another slot"): both cleanup copies call
// the same target with the method's own parameter (`data`, slot 1), so the argument slot is not
// the slot the lead initialised — the pair reads a value the statement's own value flow never
// wrote. The body still assigns the lead slot, so this is refused by the argument-slot identity
// alone.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2ArgOtherSlot {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(Object is) { closes++; }
    public Result test(byte[] data) throws IOException {
        InputStream inputStream = null;
        try {
            inputStream = getInputStream(data);
            decode(inputStream);
            return new Result(400);
        } finally {
            closeQuietly(data);
        }
    }
    public static void main(String[] args) throws Exception {
        Tf2ArgOtherSlot t = new Tf2ArgOtherSlot();
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes);
    }
}
