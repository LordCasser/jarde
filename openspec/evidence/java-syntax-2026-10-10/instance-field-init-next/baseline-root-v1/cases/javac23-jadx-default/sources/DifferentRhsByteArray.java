package defpackage;

/* JADX INFO: loaded from: instance-field-init-fixtures.jar:DifferentRhsByteArray.class */
public class DifferentRhsByteArray {
    static String trace = "";
    byte[] bytes = {mark(31)};

    static byte mark(int i) {
        trace += "eval:" + i + ";";
        return (byte) i;
    }

    public DifferentRhsByteArray() {
        trace += "body:noarg;";
    }

    public DifferentRhsByteArray(int i) {
        trace += "body:int:" + i + ";";
    }
}
