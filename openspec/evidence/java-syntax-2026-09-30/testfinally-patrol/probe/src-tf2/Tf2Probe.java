// The same-layout probe whose body can fail after the assignment, whose `getInputStream` can
// return null, and whose cleanup can itself fail: the three variable paths the certificate's
// behavior claim covers. The flags are static so the reflection runner can drive every path
// against each recompiled side.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf2Probe {
    public static boolean failCleanup = false;

    public Result test(byte[] data) throws Exception {
        InputStream inputStream = null;
        try {
            inputStream = getInputStream(data);
            decode(inputStream);
            return new Result(400);
        } finally {
            closeQuietly(inputStream);
        }
    }

    private InputStream getInputStream(byte[] data) throws Exception {
        if (Support.failQuery) {
            return null;
        }
        return new ByteArrayInputStream(data);
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

class Support {
    public static boolean failBody = false;
    public static boolean failQuery = false;
    public static int closes = 0;

    static int available(InputStream inputStream) throws Exception {
        if (failBody) {
            throw new RuntimeException("body");
        }
        return inputStream == null ? 0 : inputStream.available();
    }
}
