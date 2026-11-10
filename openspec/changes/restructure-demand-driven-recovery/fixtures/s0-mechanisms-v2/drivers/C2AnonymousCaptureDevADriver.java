public class C2AnonymousCaptureDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            C2AnonymousCaptureDevA.events = "";
            try { System.out.println(n + "|" + C2AnonymousCaptureDevA.run(n) + "|" + C2AnonymousCaptureDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C2AnonymousCaptureDevA.events); }
        }
    }
}
