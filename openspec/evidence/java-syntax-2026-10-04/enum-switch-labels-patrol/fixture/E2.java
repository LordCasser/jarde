public class E2 {
    public static String byColor(Color c) {
        switch (c) {
            case RED: return "r";
            case GREEN: return "g";
            default: return "?";
        }
    }
    public static int twice(Color c) {
        int n = 0;
        switch (c) { case BLUE: n += 10; break; default: n += 1; }
        switch (c) { case RED: n += 100; break; default: n += 5; }
        return n;
    }
    public static void main(String[] a) {
        System.out.println(byColor(Color.RED) + byColor(Color.GREEN) + byColor(Color.BLUE));
        System.out.println(twice(Color.BLUE) + ":" + twice(Color.RED));
    }
}
