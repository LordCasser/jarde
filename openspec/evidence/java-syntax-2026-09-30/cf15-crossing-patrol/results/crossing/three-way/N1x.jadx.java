package defpackage;

import java.io.ByteArrayOutputStream;

/* JADX INFO: loaded from: N1x.class */
public class N1x {
    public static ByteArrayOutputStream open() {
        return new ByteArrayOutputStream();
    }

    public static void t(int i) {
        if (i != 0) {
            throw new IllegalStateException("state");
        }
    }

    public static void main(String[] strArr) {
        run(strArr.length);
    }

    public static void run(int i) {
        ByteArrayOutputStream byteArrayOutputStreamOpen = open();
        try {
            t(i);
            byteArrayOutputStreamOpen.write(1);
        } catch (IllegalStateException e) {
            System.out.println(byteArrayOutputStreamOpen.size());
        }
        System.out.println("end:" + byteArrayOutputStreamOpen.size());
    }
}
