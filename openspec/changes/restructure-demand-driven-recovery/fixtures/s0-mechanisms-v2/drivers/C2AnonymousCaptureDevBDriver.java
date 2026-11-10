public class C2AnonymousCaptureDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            C2AnonymousCaptureDevB.trace = "";
            try { System.out.println(n + "|" + C2AnonymousCaptureDevB.run(n) + "|" + C2AnonymousCaptureDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C2AnonymousCaptureDevB.trace); }
        }
    }
}
