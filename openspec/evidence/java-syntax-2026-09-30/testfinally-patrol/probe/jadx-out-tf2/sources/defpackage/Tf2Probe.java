package defpackage;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

/* JADX INFO: loaded from: Tf2Probe.class */
public class Tf2Probe {
    public static boolean failCleanup = false;

    public Result test(byte[] bArr) throws Exception {
        InputStream inputStream = null;
        try {
            InputStream inputStream2 = getInputStream(bArr);
            decode(inputStream2);
            return new Result(400);
        } finally {
            closeQuietly(inputStream);
        }
    }

    private InputStream getInputStream(byte[] bArr) throws Exception {
        if (Support.failQuery) {
            return null;
        }
        return new ByteArrayInputStream(bArr);
    }

    private int decode(InputStream inputStream) throws Exception {
        return Support.available(inputStream);
    }

    private void closeQuietly(InputStream inputStream) {
        Support.closes++;
        if (failCleanup) {
            throw new RuntimeException("cleanup");
        }
    }
}
