package defpackage;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

/* JADX INFO: loaded from: Tf3.class */
public class Tf3 {
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
        return new byte[0];
    }

    private boolean validate() throws Exception {
        return false;
    }

    private InputStream getInputStream() throws Exception {
        return new ByteArrayInputStream(new byte[0]);
    }

    private byte[] read(InputStream inputStream) throws Exception {
        return new byte[0];
    }

    private static void close(InputStream inputStream) {
    }
}
