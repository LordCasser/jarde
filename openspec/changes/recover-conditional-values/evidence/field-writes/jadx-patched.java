package defpackage;

/* JADX INFO: loaded from: ConditionalFieldWritesNon01.class */
public final class ConditionalFieldWrites {
    public static boolean staticFlag;
    public boolean instanceFlag;
    public int integerField;

    /* JADX WARN: Multi-variable type inference failed */
    /* JADX WARN: Type inference failed for: r0v1 */
    /* JADX WARN: Type inference failed for: r0v3 */
    public static void putStatic(boolean choose) {
        staticFlag = choose ? 2 : 3;
    }

    /* JADX WARN: Multi-variable type inference failed */
    /* JADX WARN: Type inference failed for: r1v1 */
    /* JADX WARN: Type inference failed for: r1v3 */
    public void putInstance(boolean choose) {
        this.instanceFlag = choose ? 2 : 3;
    }

    public void putInteger(boolean choose) {
        this.integerField = choose ? 2 : 3;
    }

    public static void main(String[] args) {
        ConditionalFieldWrites value = new ConditionalFieldWrites();
        putStatic(false);
        value.putInstance(true);
        System.out.println(staticFlag + ":" + value.instanceFlag);
        putStatic(true);
        value.putInstance(false);
        System.out.println(staticFlag + ":" + value.instanceFlag);
    }
}
