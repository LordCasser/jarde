// Patch base (task 1.2, "the two copies call different targets"): the fixed lowering itself,
// compiled with an unused same-descriptor `closeOther` so the bytecode patch can point the
// handler copy's `invokespecial` at it. The normal copy stays on `closeQuietly`; after the
// patch the two copies call different targets and the fold would drop or invent a call.
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public class Tf2TargetMismatch {
    public static final class Result { private final int mCode; public Result(int code) { mCode = code; } public int getCode() { return mCode; } }
    static int closes = 0;
    private InputStream getInputStream(byte[] data) throws IOException { return new ByteArrayInputStream(data); }
    private int decode(InputStream inputStream) throws IOException { return inputStream.available(); }
    private void closeQuietly(InputStream is) { closes++; }
    private void closeOther(InputStream is) { closes += 1000; }
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
        Tf2TargetMismatch t = new Tf2TargetMismatch();
        t.closeOther(null);  // referenced so the pool carries the methodref the patch points at
        System.out.println("value=" + t.test(new byte[1]).getCode() + " closes=" + closes);
    }
}
