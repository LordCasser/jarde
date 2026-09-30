// Negative 2 (task 1.2, "lead 非 null 初始化"): the statement's lead initialises the cleanup
// local from a static field instead of `aconst_null`, so the lead is a `getstatic; astore`
// pair and the copies' arguments no longer read the lead's own `null`. The exception table and
// the body's layout stay the fixed two-row lowering's.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf3LeadField {
    public byte[] bytes;
    static java.io.InputStream preset;

    public byte[] test() throws Exception {
        InputStream inputStream = preset;
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
        }
    }

    private byte[] convert(byte[] b) throws Exception { return new byte[0]; }
    private boolean validate() throws Exception { return false; }
    private InputStream getInputStream() throws Exception { return new ByteArrayInputStream(new byte[] {}); }
    private byte[] read(InputStream in) throws Exception { return new byte[] {}; }
    private static void close(InputStream is) { }
}
