class VSib {
    static class Base { int id() { return 3; } }
    static class Err extends Exception { }
    static class Kid extends Base { }
    static Kid kid;
    static Base make() throws Err { return new Kid(); }
    public static void main(String[] a) throws Err { System.out.println(make().id()); }
}
