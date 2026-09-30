import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
public class Tf2 {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { }
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
}
