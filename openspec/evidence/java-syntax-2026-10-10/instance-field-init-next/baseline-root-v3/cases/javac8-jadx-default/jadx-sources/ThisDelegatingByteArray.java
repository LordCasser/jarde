package defpackage;

/* JADX INFO: loaded from: instance-field-init-fixtures.jar:ThisDelegatingByteArray.class */
public class ThisDelegatingByteArray {
    static String trace = "";
    byte[] bytes;

    static byte mark(int i) {
        trace += "eval:" + i + ";";
        return (byte) i;
    }

    public ThisDelegatingByteArray() {
        this(7);
        trace += "body:delegate;";
    }

    public ThisDelegatingByteArray(int i) {
        this.bytes = new byte[]{mark(21), mark(22)};
        trace += "body:target:" + i + ";";
    }
}
