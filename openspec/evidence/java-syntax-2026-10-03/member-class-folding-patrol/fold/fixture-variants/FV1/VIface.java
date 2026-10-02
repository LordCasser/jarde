class VIface {
    interface Mark { int TAG = 5; }
    static class Tagged implements Mark { int tag() { return TAG; } }
    public static void main(String[] a) { System.out.println(new Tagged().tag()); }
}
