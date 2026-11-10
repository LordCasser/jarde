public class T1RefMergeDev {
    static String trace = "";
    interface Label { String label(); }
    static class North implements Label { public String label() { return "N"; } }
    static class South implements Label { public String label() { return "S"; } }
    public static String run(int n) {
        trace = "";
        String out;
        { Label slot = n < 1 ? new North() : new South(); out = slot.label(); }
        { int slot = n + 1; trace += slot; }
        return out + ":" + trace;
    }
}
