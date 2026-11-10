public class R2LabeledExitDevA {
    static String events = "";
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";
 int accumulator = 0;
        try {
            outer: for (int outerIndex = 0; outerIndex < 3; outerIndex++) {
                for (int innerIndex = 0; innerIndex < 4; innerIndex++) {
                    events += outerIndex + "" + innerIndex;
                    if (inputValue == 0 && innerIndex == 1) continue outer;
                    if (inputValue == 1 && outerIndex == 1) break outer;
                    if (inputValue == 4 && outerIndex == 1 && innerIndex == 2) return "ret:" + events;
                    if (inputValue == 9 && outerIndex == 1 && innerIndex == 1) throw new IllegalArgumentException();
                    accumulator += outerIndex + innerIndex;
                }
            }
        } catch (IllegalArgumentException failure) { events += "caught"; return "throw:" + events; }
        return accumulator + ":" + events;
    }
}
