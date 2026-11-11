package dt14;

public final class CustomInitRunner {
    public static void main(String[] args) {
        System.out.println("map=" + CustomInit.BY_NAME.size() + ":"
                + (CustomInit.BY_NAME.get("RED") == CustomInit.RED) + ":"
                + (CustomInit.BY_NAME.get("BLUE") == CustomInit.BLUE));
    }
}
