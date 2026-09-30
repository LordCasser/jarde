

import java.io.ByteArrayInputStream;
import java.io.InputStream;

/* JADX INFO: loaded from: Tf3Probe.class */
public class Tf3Probe {
    public static boolean failCleanup = false;
    public byte[] bytes;

    public byte[] test() throws Exception {
        InputStream inputStream;
        InputStream inputStream2 = null;
        try {
            if (this.bytes == null) {
                if (!validate()) {
                    return null;
                }
                inputStream = getInputStream();
                this.bytes = read(inputStream);
            }
            inputStream2 = inputStream;
            return convert(this.bytes);
        } finally {
            close(inputStream);
        }
    }

    private byte[] convert(byte[] bArr) throws Exception {
        return bArr;
    }

    private boolean validate() throws Exception {
        return !Support.failValidate;
    }

    private InputStream getInputStream() throws Exception {
        if (Support.failBody) {
            throw new RuntimeException("body");
        }
        return new ByteArrayInputStream(Support.preset());
    }

    private byte[] read(InputStream inputStream) throws Exception {
        return Support.read(inputStream);
    }

    private static void close(InputStream inputStream) {
        Support.closes++;
        if (failCleanup) {
            throw new RuntimeException("cleanup");
        }
    }
}
