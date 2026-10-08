public class NoStand extends java.lang.Object {
    public static java.lang.Runnable make(java.lang.Thread arg0) {
        return arg0::start;
    }
}