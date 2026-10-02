package defpackage;

/* JADX INFO: loaded from: MV1.jar:MV1.class */
public class MV1 {

    /* JADX INFO: loaded from: MV1.jar:MV1$Inner.class */
    class Inner {
        Inner() {
        }

        int v() {
            return 9;
        }
    }

    int run() {
        return new Inner().v();
    }

    public static void main(String[] strArr) {
        System.out.println(new MV1().run());
    }
}
