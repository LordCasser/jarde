package defpackage;

/* JADX INFO: loaded from: SCGE.class */
public class SCGE<T extends java.lang.Comparable<T>> extends defpackage.SCGEBase {
    private java.util.List<T> values = new java.util.ArrayList();

    public static void main(java.lang.String[] strArr) {
        if (new defpackage.SCGE().values != null) {
            java.lang.System.out.println("values:true");
        } else {
            java.lang.System.out.println("values:false");
        }
    }
}
