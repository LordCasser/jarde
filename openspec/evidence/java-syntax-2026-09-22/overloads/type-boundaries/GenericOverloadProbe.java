public class GenericOverloadProbe {
    public static int choose(Object value) {
        return 1;
    }

    public static int choose(String value) {
        return 2;
    }

    public static int genericObjectCast() {
        return choose((Object) GenericFactory.make());
    }
}
