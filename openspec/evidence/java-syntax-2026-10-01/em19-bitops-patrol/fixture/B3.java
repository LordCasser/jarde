public class B3 {
    public static boolean twoBits(int v) { return (v & 1) != 0 && (v & 2) != 0; }
    public static boolean mixed(int v) { return (v & 1) != 0 && v > 5; }
    public static boolean twoCmp(int v) { return v > 1 && v < 9; }
    public static boolean threeBits(int v) { return (v & 1) != 0 || ((v & 2) != 0 && (v & 4) != 0); }
    public static String use(int v) { return twoBits(v) + ":" + mixed(v) + ":" + twoCmp(v) + ":" + threeBits(v); }
    public static void main(String[] a) { System.out.println(use(0x1B)); System.out.println(use(0x01)); }
}
