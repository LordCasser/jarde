// Negative 1 (task 1.2, "间隙含额外语句 / 副本文法增删"): the finally clause carries a second
// statement after the close, so the early return's gap is five instructions of call, call and
// `areturn` — not the exact four-instruction `[aload s; invoke; aload v; areturn]` the
// segmented null-lead certificate proves.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf3GapExtra {
    public byte[] bytes;
    static int closes = 0;

    public byte[] test() throws Exception {
        InputStream inputStream = null;
        try {
            if (bytes == null) {
                if (!validate()) {
                    return null;
                }
                inputStream = getInputStream();
                bytes = read(inputStream);
            }
            return convert(bytes);
        } finally {
            close(inputStream);
            count();
        }
    }

    private byte[] convert(byte[] b) throws Exception { return new byte[0]; }
    private boolean validate() throws Exception { return false; }
    private InputStream getInputStream() throws Exception { return new ByteArrayInputStream(new byte[] {}); }
    private byte[] read(InputStream in) throws Exception { return new byte[] {}; }
    private static void close(InputStream is) { closes++; }
    private static void count() { }

    public static void main(String[] args) throws Exception {
        Tf3GapExtra t = new Tf3GapExtra();
        System.out.println("value=" + t.test() + " closes=" + closes);
    }
}
