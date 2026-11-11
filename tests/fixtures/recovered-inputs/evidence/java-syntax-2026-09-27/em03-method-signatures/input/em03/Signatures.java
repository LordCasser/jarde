package em03;

import java.io.IOException;

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
