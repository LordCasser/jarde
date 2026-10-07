/// The anchor's own certificate shape over a stream the **caller** owns: `countLines`'s guard, its
/// loop and its saved return, with the resource handed in instead of constructed.
///
/// This is the mid-read leg's method. A path-based method cannot fail *inside* its protected range
/// on this platform — `new FileInputStream(directory)` throws at construction, before the range —
/// so the driver hands this method a reader whose third read throws, and the `finally`'s close is
/// observable through that reader: the shape is the certificate's own, and the close's timing is
/// measured rather than assumed.
public final class IOMidRead {
    private IOMidRead() {}

    static int countRemaining(java.io.BufferedReader source) throws java.io.IOException {
        java.io.BufferedReader r = source;
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
