class VInner {
    class Member { int v() { return 4; } }
    int run() { return new Member().v(); }
    public static void main(String[] a) { System.out.println(new VInner().run()); }
}
