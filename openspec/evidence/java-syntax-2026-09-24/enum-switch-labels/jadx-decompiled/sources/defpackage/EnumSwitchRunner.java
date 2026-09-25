package defpackage;

/* JADX INFO: loaded from: enum-switch.jar:EnumSwitchRunner.class */
public class EnumSwitchRunner {
    public static void main(String[] strArr) {
        for (Hue hue : new Hue[]{Hue.RED, Hue.BLUE, Hue.GREEN}) {
            EnumSwitchSubject.reset();
            System.out.println(EnumSwitchSubject.choose(hue) + "|" + EnumSwitchSubject.trace());
        }
        EnumSwitchSubject.reset();
        try {
            EnumSwitchSubject.choose(null);
            System.out.println("unexpected|" + EnumSwitchSubject.trace());
        } catch (NullPointerException e) {
            System.out.println("null|" + EnumSwitchSubject.trace());
        }
    }
}
