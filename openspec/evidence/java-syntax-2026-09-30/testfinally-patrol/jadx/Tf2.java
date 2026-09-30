package defpackage;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

/* JADX INFO: loaded from: Tf2.class */
public class Tf2 {
    private InputStream getInputStream(byte[] bArr) throws IOException {
        return new ByteArrayInputStream(bArr);
    }

    private int decode(InputStream inputStream) throws IOException {
        return inputStream.available();
    }

    private void closeQuietly(InputStream inputStream) {
    }

    public Result test(byte[] bArr) throws IOException {
        InputStream inputStream = null;
        try {
            InputStream inputStream2 = getInputStream(bArr);
            decode(inputStream2);
            return new Result(400);
        } finally {
            closeQuietly(inputStream);
        }
    }
}
