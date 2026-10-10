public final class CharProducerIntOverload {
    static String render(String text) {
        int value = text.charAt(0);
        value = 46;
        return new StringBuilder().append(value).toString();
    }

    public static void main(String[] args) {
        System.out.println(render("x"));
    }
}
