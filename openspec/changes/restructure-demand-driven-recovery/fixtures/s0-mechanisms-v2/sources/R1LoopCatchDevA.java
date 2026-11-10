public class R1LoopCatchDevA {
    static String events = "";
    static int step(int sample) { events += "s"; if (sample % 3 == 0) throw new IllegalArgumentException(); return sample * 2; }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";
 int accumulator = 0;
        for (int iteration = 1; iteration <= Math.abs(inputValue) % 5 + 1; iteration++) {
            try { accumulator += step(inputValue + iteration); events += "o"; }
            catch (IllegalArgumentException failure) { events += "c"; accumulator--; }
        }
        return accumulator + ":" + events;
    }
}
