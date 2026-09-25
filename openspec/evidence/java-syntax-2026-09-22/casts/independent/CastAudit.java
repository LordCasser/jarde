public class CastAudit {
    static Object value;
    static int calls;
    Object field = "member";
    static Object tick() { calls = calls + 1; return value; }
    public static String direct(Object x) { return (String)x; }
    public static int receiver(Object x) { return ((String)x).length(); }
    public static int array(Object x, int i) { return ((int[])x)[i]; }
    public static String[][] multi(Object x) { return (String[][])x; }
    public static Number nested(Object x) { return (Number)(Runnable)x; }
    public static String staticField() { return (String)value; }
    public String instanceField() { return (String)this.field; }
    public static String local(Object x) { String s = (String)x; return s; }
    public static String callCast() { return (String)tick(); }
    public static Object unusedLocal(Object x) { String s = (String)x; return tick(); }
}
