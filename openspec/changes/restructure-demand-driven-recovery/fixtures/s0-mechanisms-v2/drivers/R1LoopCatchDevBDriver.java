public class R1LoopCatchDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            R1LoopCatchDevB.trace = "";
            try { System.out.println(n + "|" + R1LoopCatchDevB.run(n) + "|" + R1LoopCatchDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R1LoopCatchDevB.trace); }
        }
    }
}
