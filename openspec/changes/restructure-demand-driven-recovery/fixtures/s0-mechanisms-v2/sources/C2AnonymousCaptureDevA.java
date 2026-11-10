public class C2AnonymousCaptureDevA {
    static String events = "";
    interface Task { String run(); }
    static Task make(int capturedInput, boolean extended) {
        if (extended) return new Task() { { events += "L"; } public String run() { events += "l"; return "long" + capturedInput; } };
        return new Task() { { events += "S"; } public String run() { events += "s"; return "short" + capturedInput; } };
    }
    public static String run(int inputValue) {
        events = "";
        long padding64 = ((long) inputValue << 32) ^ (((long) inputValue) & 0xffffL);
        events += "A" + padding64 + ";";
 final int seedValue = inputValue + 5;
        Task firstJob = make(seedValue, inputValue % 2 == 0), secondJob = make(seedValue + 1, inputValue < 0);
        return firstJob.run() + ":" + secondJob.run() + ":" + events;
    }
}
