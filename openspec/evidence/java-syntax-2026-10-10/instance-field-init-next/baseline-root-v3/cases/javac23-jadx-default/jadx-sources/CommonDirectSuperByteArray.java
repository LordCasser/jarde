package defpackage;

/* JADX INFO: loaded from: instance-field-init-fixtures.jar:CommonDirectSuperByteArray.class */
public class CommonDirectSuperByteArray {
    static String trace = "";
    byte[] bytes = {mark(10), mark(20)};

    static byte mark(int i) {
        trace += "eval:" + i + ";";
        return (byte) i;
    }

    public CommonDirectSuperByteArray() {
        trace += "body:noarg;";
    }

    public CommonDirectSuperByteArray(int i) {
        trace += "body:int:" + i + ";";
    }
}
