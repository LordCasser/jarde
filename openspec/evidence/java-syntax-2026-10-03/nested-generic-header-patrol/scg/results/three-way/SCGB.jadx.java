package defpackage;

/* JADX INFO: loaded from: SCGB.class */
public class SCGB<T extends java.lang.Comparable<T>> {
    private java.util.Map<java.lang.String, java.util.List<T>> index = new java.util.HashMap();

    public static void main(java.lang.String[] strArr) {
        defpackage.SCGB scgb = new defpackage.SCGB();
        if (scgb.index != null) {
            java.lang.System.out.println("index:true");
        } else {
            java.lang.System.out.println("index:false");
        }
        if (scgb.index instanceof java.util.Map) {
            java.lang.System.out.println("read:true");
        } else {
            java.lang.System.out.println("read:false");
        }
    }
}
