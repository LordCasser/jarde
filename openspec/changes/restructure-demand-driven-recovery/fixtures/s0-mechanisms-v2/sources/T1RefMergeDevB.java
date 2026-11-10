public class T1RefMergeDevB {
    static String trace = "";
    interface Label { String label(); }
    static class North implements Label { public String label() { return "N"; } }
    static class South implements Label { public String label() { return "S"; } }
    public static String run(int n) {
        trace = "";
        trace += "B";
        String out;
        { { Label slot = n < 1 ? new North() : new South(); out = slot.label() + ":" + slot.label().length(); } }
        { int slot = n + 1; trace += slot; }
        trace += "b";
        return out + ":" + trace;
    }
}
