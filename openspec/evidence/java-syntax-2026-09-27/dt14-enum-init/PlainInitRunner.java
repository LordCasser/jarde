package dt14;

public final class PlainInitRunner {
    public static void main(String[] args) {
        System.out.println("plain=" + PlainInit.values().length + ":"
                + (PlainInit.valueOf("BLUE") == PlainInit.BLUE));
    }
}
