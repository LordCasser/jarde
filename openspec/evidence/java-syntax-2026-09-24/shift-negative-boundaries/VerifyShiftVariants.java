import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.security.MessageDigest;
import java.util.Arrays;

/** Defines descriptor-only variants from memory so the JVM, not a parser, decides validity. */
public final class VerifyShiftVariants {
    private static final class BytesLoader extends ClassLoader {
        Class<?> define(byte[] bytes) {
            return defineClass("ShiftSlice", bytes, 0, bytes.length);
        }
    }

    private static int u2(byte[] bytes, int at) {
        return ((bytes[at] & 255) << 8) | (bytes[at + 1] & 255);
    }

    private static void putU2(byte[] bytes, int at, int value) {
        bytes[at] = (byte) (value >>> 8);
        bytes[at + 1] = (byte) value;
    }

    private static byte[] replaceUtf8(byte[] classFile, String oldValue, String newValue)
            throws Exception {
        byte[] bytes = classFile.clone();
        byte[] oldBytes = oldValue.getBytes("UTF-8");
        byte[] newBytes = newValue.getBytes("UTF-8");
        if (oldBytes.length != newBytes.length) throw new AssertionError("length changed");
        int count = u2(bytes, 8);
        int at = 10;
        int matches = 0;
        for (int index = 1; index < count; index++) {
            int tag = bytes[at++] & 255;
            switch (tag) {
                case 1: {
                    int length = u2(bytes, at);
                    at += 2;
                    if (length == oldBytes.length
                            && Arrays.equals(Arrays.copyOfRange(bytes, at, at + length), oldBytes)) {
                        System.arraycopy(newBytes, 0, bytes, at, length);
                        matches++;
                    }
                    at += length;
                    break;
                }
                case 3: case 4: at += 4; break;
                case 5: case 6: at += 8; index++; break;
                case 7: case 8: case 16: case 19: case 20: at += 2; break;
                case 9: case 10: case 11: case 12: case 17: case 18: at += 4; break;
                case 15: at += 3; break;
                default: throw new AssertionError("unknown constant pool tag " + tag);
            }
        }
        if (matches != 1) throw new AssertionError("expected one descriptor constant, got " + matches);
        return bytes;
    }

    private static String sha256(byte[] bytes) throws Exception {
        byte[] digest = MessageDigest.getInstance("SHA-256").digest(bytes);
        StringBuilder out = new StringBuilder();
        for (byte value : digest) out.append(String.format("%02x", value & 255));
        return out.toString();
    }

    private static Class<?> verify(byte[] bytes, String label) throws Exception {
        Class<?> type = new BytesLoader().define(bytes);
        // Resolving and invoking forces verification of the actual defined bytes under -Xverify:all.
        type.getDeclaredMethods();
        System.out.println(label + " sha256=" + sha256(bytes) + " verified=" + type.getName());
        return type;
    }

    public static void main(String[] args) throws Exception {
        byte[] original = Files.readAllBytes(Paths.get(args[0]));
        verify(original, "original");

        Class<?> booleanLeft = verify(replaceUtf8(original, "(SI)I", "(ZI)I"), "boolean-left");
        Method left = booleanLeft.getMethod("shortLeft", boolean.class, int.class);
        System.out.println("boolean-left result=" + left.invoke(null, true, 2));

        Class<?> booleanDistance = verify(replaceUtf8(original, "(II)I", "(IZ)I"), "boolean-distance");
        Method distance = booleanDistance.getMethod("intLeft", int.class, boolean.class);
        System.out.println("boolean-distance result=" + distance.invoke(null, 3, true));

        Class<?> booleanConsumer = verify(replaceUtf8(original, "(II)I", "(II)Z"), "boolean-consumer");
        Method consumer = booleanConsumer.getMethod("intLeft", int.class, int.class);
        System.out.println("boolean-consumer result=" + consumer.invoke(null, 3, 1));
    }
}
