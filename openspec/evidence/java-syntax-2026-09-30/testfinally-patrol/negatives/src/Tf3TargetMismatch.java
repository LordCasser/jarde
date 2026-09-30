// Patch base for the bytecode-patched negative `Tf3TargetMismatch`: the fixed Tf3 lowering
// itself, compiled fresh so `patch-tf3.py` can patch the handler copy's call bytes in place.
// The extra `closeOther` is the unused same-descriptor static the patch retargets the handler
// copy's `invokestatic` to, so the three copies no longer share one target.
import java.io.ByteArrayInputStream;
import java.io.InputStream;

public class Tf3TargetMismatch {
    public byte[] bytes;

    public byte[] test() throws Exception {
        InputStream inputStream = null;
        try {
            if (bytes == null) {
                if (!validate()) {
                    return null;
                }
                inputStream = getInputStream();
                bytes = read(inputStream);
            }
            return convert(bytes);
        } finally {
            close(inputStream);
        }
    }

    private byte[] convert(byte[] b) throws Exception { return new byte[0]; }
    private boolean validate() throws Exception { return false; }
    private InputStream getInputStream() throws Exception { return new ByteArrayInputStream(new byte[] {}); }
    private byte[] read(InputStream in) throws Exception { return new byte[] {}; }
    private static void close(InputStream is) { }
    static void closeOther(InputStream is) { }

    public static void main(String[] args) throws Exception {
        // `closeOther` must exist in the constant pool as a methodref for the patch to
        // retarget the handler copy's call at: this never-taken branch is the one reference.
        if (args.length > 99) {
            closeOther(null);
        }
        Tf3TargetMismatch t = new Tf3TargetMismatch();
        System.out.println("value=" + t.test());
    }
}
