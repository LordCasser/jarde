public class T1RefMergeHoldout {
    static String trace = "";
    interface View { String text(); }
    static class Token implements View { final int id; Token(int id) { this.id = id; } public String text() { return "T" + id; } }
    static class Word implements View { final String value; Word(String v) { value = v; } public String text() { return value; } }
    public static String run(int n) {
        trace = "";
        View current = n == 0 ? null : (n < 0 ? new Token(-n) : new Word("W" + n));
        String first = current == null ? "nil" : current.text();
        for (int k = 0; k < n % 3 + 3; k++) { current = new Token(k); trace += current.text(); }
        return first + ":" + current.text() + ":" + trace;
    }
}
