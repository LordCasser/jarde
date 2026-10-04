package defpackage;

/* JADX INFO: loaded from: MN.class */
public class MN {
    static MN.Hold<java.lang.String> f1 = new MN.Hold<>("a");
    static MN.Hold<java.lang.String> f2 = new MN.Hold<>("b");
    MN.Hold<java.lang.Integer> f3 = new MN.Hold<>(5);

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(((java.lang.String) f1.v) + ((java.lang.String) f2.v) + new defpackage.MN().f3.v);
    }
}
