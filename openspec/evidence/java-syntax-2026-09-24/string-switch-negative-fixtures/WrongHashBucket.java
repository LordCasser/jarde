public class WrongHashBucket {
    static int calls;

    static String read(String value) {
        calls++;
        return value;
    }

    public static int choose(String value) {
        String selected = read(value);
        int hash = selected.hashCode();
        int discriminator = -1;
        switch (hash) {
            case 96354:
                if (selected.equals("Aa")) discriminator = 0;
                break;
            case 2112:
                if (selected.equals("abc")) discriminator = 1;
                break;
        }
        switch (discriminator) {
            case 0: return 10;
            case 1: return 20;
            default: return 40;
        }
    }
}
