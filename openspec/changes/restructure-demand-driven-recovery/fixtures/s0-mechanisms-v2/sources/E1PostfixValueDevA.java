public class E1PostfixValueDevA {
    static String events = "";
    static int take(int cells, int rightValue) { events += "t"; return cells * 10 + rightValue; }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";

        int cursor = inputValue, previous = cursor++, advanced = ++cursor;
        int[] cells = new int[2];
        int saved = (cells[0] = take(previous, cursor++));
        int aggregate = (cells[1] = cursor--) + saved;
        return previous + ":" + advanced + ":" + cursor + ":" + aggregate + ":" + events;
    }
}
