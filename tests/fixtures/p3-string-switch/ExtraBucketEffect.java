public class ExtraBucketEffect {
    static int calls;

    public static int choose(String value) {
        String selector = value;
        int discriminator = -1;
        switch (selector.hashCode()) {
            case 2112:
                calls++;
                if (selector.equals("Aa")) {
                    discriminator = 0;
                }
                break;
            default:
                break;
        }
        switch (discriminator) {
            case 0:
                return 10;
            default:
                return 20;
        }
    }
}
