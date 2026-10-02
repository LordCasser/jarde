public class M1 {
    static class Base { void hi() { System.out.println("hi"); } }
    interface Ctrl { }
    static class Deep extends Base implements Ctrl { }
    static class Err extends Exception { }
    static class Inner { static class Leaf extends Base { } }
    static Deep deepField;
    Base baseField;
    void work() throws Err, RuntimeException { }
    static Deep make() throws Err { return new Deep(); }
    public static void main(String[] a) throws Err {
        M1 m = new M1();
        m.baseField = new Deep();
        m.baseField.hi();
        m.work();
        System.out.println("ok");
    }
}
class M1User extends M1.Base implements M1.Ctrl {
    public static void run() { new M1User().hi(); }
}
