public class T2GenericRelayDevA {
    static String events = "";
    interface Sink<T> { String accept(T itemValue); }
    static class Holder<T> { final T itemValue; Holder(T v) { itemValue = v; } T get() { return itemValue; } }
    static class Relay<T> { final Sink<T> consumer; Relay(Sink<T> textValue) { consumer = textValue; } String send(Holder<T> holder) { return consumer.accept(holder.get()); } }
    static String mark(String textValue) { events += "m"; return textValue + "!"; }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";

        Sink<String> consumer = itemValue -> { events += "s"; return itemValue + inputValue; };
        return new Relay<String>(consumer).send(new Holder<String>(mark("v" + inputValue))) + ":" + events;
    }
}
