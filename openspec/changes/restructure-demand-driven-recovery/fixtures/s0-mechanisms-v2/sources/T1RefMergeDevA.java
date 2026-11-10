public class T1RefMergeDevA {
    static String events = "";
    interface Marker { String label(); }
    static class Northern implements Marker { public String label() { return "N"; } }
    static class Southern implements Marker { public String label() { return "S"; } }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";

        String answer;
        { Marker localValue = inputValue < 1 ? new Northern() : new Southern(); answer = localValue.label(); }
        { int localValue = inputValue + 1; events += localValue; }
        return answer + ":" + events;
    }
}
