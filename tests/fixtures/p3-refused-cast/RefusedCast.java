public class RefusedCast {
    static int calls;
    static int tick() { calls++; return 7; }
    public static String fieldCast() { return (String) External.value; }
    public static String instanceCast(External e) { return (String) e.instance; }
    public static String chainCast() { return (String) External.holder.value; }
    public static int leftRead() { return External.count + tick(); }
    public static int rightRead() { return tick() + External.count; }
}

class External {
    static Object value;
    Object instance = "instance-ok";
    static int count;
    static Holder holder;
    static { RefusedCast.calls++; value = "ok"; holder = new Holder(); holder.value = "chained"; }
}

class Holder { Object value; }
