public class B5 {
    public static String s1(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        return hasA + ":";
    }
    public static String s2(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        boolean hasB = (v & 0x04) != 0;
        return hasA + ":" + hasB;
    }
    public static boolean s3(int v) {
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        return hasA;
    }
    public static void main(String[] a) { System.out.println(s1(3)); System.out.println(s2(3)); System.out.println(s3(3)); }
}
