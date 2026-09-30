// The same-layout probe whose `validate` can refuse, whose body can fail at the stream
// acquisition, whose cleanup can itself fail, and whose `bytes` field can be preset: the five
// variable paths the certificate's behavior claim covers (cached, uncached, early `return
// null`, body throw, cleanup throw). The flags are static so the reflection runner can drive
// every path against each recompiled side.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf3Probe {
    public static boolean failCleanup = false;
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

    private byte[] convert(byte[] b) throws Exception { return b; }
    private boolean validate() throws Exception { return !Support.failValidate; }
    private InputStream getInputStream() throws Exception {
        if (Support.failBody) {
            throw new RuntimeException("body");
        }
        return new ByteArrayInputStream(Support.preset());
    }
    private byte[] read(InputStream in) throws Exception { return Support.read(in); }
    private static void close(InputStream is) {
        Support.closes++;
        if (failCleanup) {
            throw new RuntimeException("cleanup");
        }
    }
}

class Support {
    public static boolean failValidate = false;
    public static boolean failBody = false;
    public static int closes = 0;

    static byte[] preset() { return new byte[] { 1, 2 }; }

    static byte[] read(InputStream in) throws Exception {
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        byte[] buffer = new byte[8];
        for (int n = in.read(buffer); n > 0; n = in.read(buffer)) {
            out.write(buffer, 0, n);
        }
        return out.toByteArray();
    }
}
