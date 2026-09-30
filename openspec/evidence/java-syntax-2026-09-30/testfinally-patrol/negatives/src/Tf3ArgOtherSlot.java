// Patch base for the bytecode-patched negative `Tf3ArgOtherSlot`: the fixed Tf3 lowering itself,
// compiled fresh so `patch-tf3.py` can rewrite one copy, one completion, one condition or one
// exception-table row in place. The source is deliberately the exact fixed lowering.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf3ArgOtherSlot {
    public byte[] bytes;

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
        }
    }

    private byte[] convert(byte[] b) throws Exception { return new byte[0]; }
    private boolean validate() throws Exception { return false; }
    private InputStream getInputStream() throws Exception { return new ByteArrayInputStream(new byte[] {}); }
    private byte[] read(InputStream in) throws Exception { return new byte[] {}; }
    private static void close(InputStream is) { }

    public static void main(String[] args) throws Exception {
        Tf3ArgOtherSlot t = new Tf3ArgOtherSlot();
        System.out.println("value=" + t.test());
    }
}
