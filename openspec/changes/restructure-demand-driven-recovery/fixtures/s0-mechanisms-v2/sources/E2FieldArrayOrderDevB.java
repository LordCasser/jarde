public class E2FieldArrayOrderDevB {
    static String trace = "";
    static class Box { int value; int[] data = new int[4]; }
    static Box box;
    static Box receiver() { trace += "R"; return box; }
    static int index(int n) { trace += "I"; return Math.abs(n) % 4; }
    static int rhs(int n) { trace += "V"; if (n == 9) throw new IllegalArgumentException(); return n + 3; }
    public static String run(int n) {
        trace = ""; box = n == 0 ? null : new Box();
        trace += "B";
        receiver().value += rhs(n);
        int written; { written = receiver().data[index(n)] = rhs(n); }
        trace += "b";
        return box.value + ":" + box.data[Math.abs(n) % 4] + ":" + written + ":" + written + ":" + trace;
    }
}
