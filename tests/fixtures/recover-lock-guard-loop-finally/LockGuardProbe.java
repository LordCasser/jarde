/// The io-wrapping patrol's resource-across-finally shape as a **single-method probe**: one local
/// handle, a construction chain before the protected range, a loop and a saved return in the body,
/// and a release that declares a checked exception — so `javac` writes the self-protection row.
///
/// This is the shape the lock-guard certificate must **not** claim: its one invocation before the
/// range is not a field read of the receiver the copies release, and its two rows are the resource
/// lowering's, not the lock guard's single catch-all. The probe exists so this slice's boundary is
/// measured rather than assumed; the whole-class IO acceptance is another slice's (it needs the
/// widening and construction readings the patrol's README registers).
public class LockGuardProbe {
    static int countLines(String path) throws java.io.IOException {
        java.io.BufferedReader r = new java.io.BufferedReader(
                new java.io.InputStreamReader(new java.io.FileInputStream(path), "UTF-8"));
        try {
            int n = 0;
            String line;
            while ((line = r.readLine()) != null) {
                n++;
            }
            return n;
        } finally {
            r.close();
        }
    }
}
