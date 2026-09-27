package em10;

import java.io.IOException;

/* JADX INFO: loaded from: fixture.jar:em10/InvokeFixture.class */
public class InvokeFixture extends InvokeBase {
    private int receivers;

    private InvokeWorker receiver() {
        this.receivers++;
        return new InvokeWorker();
    }

    private void fail() throws IOException {
        throw new IOException("io");
    }

    long run() {
        return receiver().combine(1, 2L, 3.0d, 4) + InvokeTools.combine(5, 6L, 7.0d, 8) + inherited(9L, 10) + runCatch();
    }

    private long runCatch() {
        try {
            fail();
            return 0L;
        } catch (IOException e) {
            return receiver().combine(11, 12L, 13.0d, 14) + ((long) InvokeTools.caught(e.getMessage()).length());
        }
    }

    public static void main(String[] strArr) {
        System.out.println(new InvokeFixture().run());
    }
}
