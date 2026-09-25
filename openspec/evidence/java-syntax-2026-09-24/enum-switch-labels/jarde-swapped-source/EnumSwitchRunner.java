public class EnumSwitchRunner {
    public static void main(String[] args) {
        for (Hue hue : new Hue[] {Hue.RED, Hue.BLUE, Hue.GREEN}) {
            EnumSwitchSubject.reset();
            System.out.println(EnumSwitchSubject.choose(hue) + "|" + EnumSwitchSubject.trace());
        }
        EnumSwitchSubject.reset();
        try {
            EnumSwitchSubject.choose(null);
            System.out.println("unexpected|" + EnumSwitchSubject.trace());
        } catch (NullPointerException expected) {
            System.out.println("null|" + EnumSwitchSubject.trace());
        }
    }
}
