public class NoCheck extends java.lang.Object {
    public static java.lang.Thread make(java.lang.Thread arg0) {
        return new java.lang.Thread((java.lang.Runnable) arg0::start);
    }
}