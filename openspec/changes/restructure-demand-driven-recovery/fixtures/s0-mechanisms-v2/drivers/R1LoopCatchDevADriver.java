public class R1LoopCatchDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            R1LoopCatchDevA.events = "";
            try { System.out.println(n + "|" + R1LoopCatchDevA.run(n) + "|" + R1LoopCatchDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R1LoopCatchDevA.events); }
        }
    }
}
