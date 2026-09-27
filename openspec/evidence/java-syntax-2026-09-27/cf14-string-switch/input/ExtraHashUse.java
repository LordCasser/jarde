public class ExtraHashUse {
    static int calls;

    static String read(String value) {
        calls++;
        return value;
    }

    public static int choose(String value) {
        String selected = read(value);
        int hash = selected.hashCode();
        int parity = hash & 1;
        int discriminator = -1;
        switch (hash) {
            case 2112:
                if (selected.equals("Aa")) discriminator = 0;
                else if (selected.equals("BB")) discriminator = 1;
                break;
            case 96354:
                if (selected.equals("abc")) discriminator = 2;
                break;
        }
        int result;
        switch (discriminator) {
            case 0: result = 10; break;
            case 1: result = 20; break;
            case 2: result = 30; break;
            default: result = 40;
        }
        return result + parity;
    }
}
