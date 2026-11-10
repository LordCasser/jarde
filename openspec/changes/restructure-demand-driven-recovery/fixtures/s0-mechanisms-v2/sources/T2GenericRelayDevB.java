public class T2GenericRelayDevB {
    static String trace = "";
    interface Sink<T> { String accept(T value); }
    static class Holder<T> { final T value; Holder(T v) { value = v; } T get() { return value; } }
    static class Relay<T> { final Sink<T> sink; Relay(Sink<T> s) { sink = s; } String send(Holder<T> h) { { String first = sink.accept(h.get()); String second = sink.accept(h.get()); return first + "/" + second; } } }
    static String mark(String s) { trace += "m"; return s + "!"; }
    public static String run(int n) {
        trace = "";
        trace += "B";
        Sink<String> sink = value -> { trace += "s"; return value + n; };
        String result = new Relay<String>(sink).send(new Holder<String>(mark("v" + n)));
        trace += "b";
        return result + ":" + trace;
    }
}
