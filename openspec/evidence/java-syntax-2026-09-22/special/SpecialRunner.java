public class SpecialRunner {
    public static void main(String[] args) {
        SpecialProbe special = new SpecialProbe();
        System.out.println("value=" + special.value());
        System.out.println("defaultCall=" + special.defaultCall());
        System.out.println("own=" + special.callOwnPrivate(5));
        System.out.println("other=" + special.callOtherPrivate(new SpecialProbe(), 5));
    }
}
