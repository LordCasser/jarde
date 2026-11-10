public class T2GenericRelayDev {
    static String trace = "";
    interface Sink<T> { String accept(T value); }
    static class Holder<T> { final T value; Holder(T v) { value = v; } T get() { return value; } }
    static class Relay<T> { final Sink<T> sink; Relay(Sink<T> s) { sink = s; } String send(Holder<T> h) { return sink.accept(h.get()); } }
    static String mark(String s) { trace += "m"; return s + "!"; }
    public static String run(int n) {
        trace = "";
        Sink<String> sink = value -> { trace += "s"; return value + n; };
        return new Relay<String>(sink).send(new Holder<String>(mark("v" + n))) + ":" + trace;
    }
}
