package defpackage;

/* JADX INFO: loaded from: SG.class */
public class SG {
    private static volatile SG.Svc dcl;

    static SG.Svc holder() {
        return SG.Holder.INSTANCE;
    }

    static SG.Svc dcl() {
        if (dcl == null) {
            synchronized (defpackage.SG.class) {
                if (dcl == null) {
                    dcl = new SG.Svc();
                }
            }
        }
        return dcl;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + (holder() == holder()) + "/" + (dcl() == dcl()) + "/" + holder().bump() + "/" + dcl().bump());
    }
}
