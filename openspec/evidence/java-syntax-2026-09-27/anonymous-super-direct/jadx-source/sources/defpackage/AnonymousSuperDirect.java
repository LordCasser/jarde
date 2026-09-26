package defpackage;

/* JADX INFO: loaded from: fixture.jar:AnonymousSuperDirect.class */
public class AnonymousSuperDirect {
    private static int effects;

    private static int next() {
        effects++;
        return effects == 1 ? 7 : 2;
    }

    private static Base make() {
        return new Base(next(), next()) { // from class: AnonymousSuperDirect.1
            @Override // defpackage.Base
            int sum() {
                return super.sum() + 1;
            }
        };
    }

    public static void main(String[] strArr) {
        System.out.println(make().sum() + ":" + effects);
    }
}
