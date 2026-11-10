public class E2FieldArrayOrderDevA {
    static String events = "";
    static class Container { int payload; int[] cells = new int[4]; }
    static Container crate;
    static Container receiver() { events += "R"; return crate; }
    static int index(int inputValue) { events += "I"; return Math.abs(inputValue) % 4; }
    static int rhs(int inputValue) { events += "V"; if (inputValue == 9) throw new IllegalArgumentException(); return inputValue + 3; }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";
 crate = inputValue == 0 ? null : new Container();
        receiver().payload += rhs(inputValue);
        receiver().cells[index(inputValue)] = rhs(inputValue);
        return crate.payload + ":" + crate.cells[Math.abs(inputValue) % 4] + ":" + events;
    }
}
