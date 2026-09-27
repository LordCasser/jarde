package em03;

import java.io.IOException;

/* JADX INFO: loaded from: input.jar:em03/Signatures.class */
public class Signatures {
    public String named(String paramStr, final int number) {
        return paramStr;
    }

    public int declared(int value) throws IOException {
        return value;
    }

    public void raises() throws IOException {
        throw new IOException("negative");
    }
}
