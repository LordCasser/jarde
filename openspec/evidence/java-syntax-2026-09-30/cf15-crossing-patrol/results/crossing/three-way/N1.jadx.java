package defpackage;

import java.io.ByteArrayOutputStream;

/* JADX INFO: loaded from: N1.class */
public class N1 {
    public static ByteArrayOutputStream open() {
        return new ByteArrayOutputStream();
    }

    public static void t() {
        throw new IllegalStateException("state");
    }

    public static void main(String[] strArr) {
        ByteArrayOutputStream byteArrayOutputStreamOpen = open();
        try {
            t();
            byteArrayOutputStreamOpen.write(1);
        } catch (IllegalStateException e) {
            System.out.println(byteArrayOutputStreamOpen.size());
        }
    }
}
