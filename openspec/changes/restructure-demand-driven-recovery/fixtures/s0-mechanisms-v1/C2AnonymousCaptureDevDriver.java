public class C2AnonymousCaptureDevDriver {
 public static void main(String[] args) {
  for (int n : new int[]{-2,0,1,4,9}) {
   C2AnonymousCaptureDev.trace = "";
   try { System.out.println(n + "|" + C2AnonymousCaptureDev.run(n) + "|" + C2AnonymousCaptureDev.trace); }
   catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C2AnonymousCaptureDev.trace); }
  }
 }
}
