public class VN4 {
    static int area(Other.Box b) { return b.size(12); }
    static boolean marked(Object o) { return o instanceof Other.Mark; }
    public static void main(String[] a) {
        Other.Box b = new Other.Tagged();
        System.out.println(area(b) + ":" + marked(b) + ":" + ((Other.Mark) b).tag());
    }
}
