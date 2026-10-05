package defpackage;

/* JADX INFO: loaded from: RG.class */
public class RG {
    static final RG.Item A = new RG.Item("alpha");
    static final RG.Item B = new RG.Item("beta");
    static final RG.Item C = new RG.Item("gamma");
    static final java.util.Map<java.lang.String, RG.Item> BY_LABEL = new java.util.HashMap();

    static {
        for (RG.Item item : new RG.Item[]{A, B, C}) {
            BY_LABEL.put(item.label, item);
        }
    }

    static RG.Item of(java.lang.String str) {
        return BY_LABEL.get(str);
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + of("beta") + "/" + of("alpha") + "/" + of("?"));
    }
}
