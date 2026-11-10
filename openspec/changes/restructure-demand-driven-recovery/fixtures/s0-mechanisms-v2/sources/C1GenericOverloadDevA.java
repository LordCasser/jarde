public class C1GenericOverloadDevA {
    static String events = "";
    static String choose(String textValue) { events += "S"; return textValue.toUpperCase(); }
    static String choose(Integer numberValue) { events += "I"; return "#" + numberValue; }
    static <T extends CharSequence> T echo(T itemValue) { events += "E"; return itemValue; }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";

        String primary = choose(echo("v" + inputValue));
        String secondary = choose(Integer.valueOf(inputValue));
        return primary + ":" + secondary + ":" + events;
    }
}
