package shadow;

public class StrongCaller {
    public static String plain() {
        return ShadowPlain.<String>echo("plain");
    }

    public static String bounded() {
        return ShadowBounded.<String>echo("bounded");
    }

    public static void main(String[] args) {
        System.out.println(plain());
        System.out.println(bounded());
    }
}
